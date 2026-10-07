import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushSync } from 'svelte';
import { events } from 'fetch-event-stream';

vi.mock('fetch-event-stream', () => ({ events: vi.fn() }));
vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/chat/42') } }));
vi.mock('./http.svelte', () => ({
	APIFetch: vi.fn(),
	RawAPIFetch: vi.fn(async () => undefined),
	getError: vi.fn(() => undefined)
}));
vi.mock('./chatroom.svelte', () => ({
	getChatId: vi.fn(() => 42),
	currentRoom: { val: undefined },
	setRoomTitle: vi.fn()
}));
vi.mock('$lib/rune.svelte', () => ({ token: { value: { value: 'secret' } } }));
vi.mock('$lib/error.svelte', () => ({ displayError: vi.fn() }));

import { APIFetch, RawAPIFetch } from './http.svelte';
import { currentRoom, getChatId, setRoomTitle } from './chatroom.svelte';
import { displayError } from '$lib/error.svelte';
import {
	createMessage,
	deleteMessage,
	messages,
	paginateElement,
	pushUserMessage,
	streaming,
	syncMessage
} from './message.svelte';
import { ChatMode, FileKind, StepKind } from './types';
import type { Deep, SseResp } from './types';

const request = vi.mocked(APIFetch);
const streamRequest = vi.mocked(RawAPIFetch);
const visibilityListeners: EventListenerOrEventListenerObject[] = [];
let version = 0;

async function receive(batch: SseResp[]) {
	let finish!: () => void;
	const drained = new Promise<void>((resolve) => {
		finish = resolve;
	});
	vi.mocked(events).mockImplementationOnce(async function* () {
		try {
			for (const response of batch) yield { data: JSON.stringify(response) };
		} finally {
			finish();
		}
	});
	streamRequest.mockResolvedValueOnce(new Response());
	document.dispatchEvent(new Event('visibilitychange'));
	await drained;
}

function start(id = 101): SseResp {
	return { t: 'start', c: { id, user_msg_id: id - 1, version } };
}

function chunks() {
	const message = messages.val[0];
	if (message?.inner.t !== 'assistant') throw new Error('Expected an assistant message');
	return message.inner.c;
}

beforeAll(() => {
	const addListener = vi.spyOn(document, 'addEventListener');
	flushSync();
	for (const [event, listener] of addListener.mock.calls) {
		if (event === 'visibilitychange') visibilityListeners.push(listener);
	}
	addListener.mockRestore();
});

afterAll(() => {
	for (const listener of visibilityListeners) {
		document.removeEventListener('visibilitychange', listener);
	}
	vi.unstubAllGlobals();
});

beforeEach(async () => {
	vi.clearAllMocks();
	request.mockReset();
	streamRequest.mockReset().mockResolvedValue(undefined);
	vi.mocked(events).mockReset();
	vi.mocked(getChatId).mockReturnValue(42);
	currentRoom.val = undefined;
	messages.val = [];
	streaming.val = false;
	Object.defineProperty(document, 'visibilityState', { configurable: true, value: 'visible' });
	await receive([{ t: 'version', c: ++version }]);
	vi.clearAllMocks();
});

afterEach(() => {
	paginateElement.val = undefined;
	flushSync();
	vi.unstubAllGlobals();
});

describe('message pagination', () => {
	function mountPagination() {
		vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => {
			queueMicrotask(() => callback(0));
			return 1;
		});
		const target = document.createElement('div');
		Object.defineProperty(target, 'clientHeight', { value: 100 });
		paginateElement.val = target;
		flushSync();
		return target;
	}

	function page(id: number) {
		return {
			list: [
				{ id, token_count: 0, price: 0, inner: { t: 'user', c: { text: String(id), files: [] } } }
			]
		};
	}

	it('loads older pages until the viewport is filled or the server returns an empty page', async () => {
		request
			.mockResolvedValueOnce(page(30))
			.mockResolvedValueOnce(page(20))
			.mockResolvedValueOnce({ list: [] });
		mountPagination();

		await vi.waitFor(() => expect(request).toHaveBeenCalledTimes(3));
		expect(messages.val.map((message) => message.id)).toEqual([30, 20]);
		expect(request.mock.calls.map(([options]) => options.body)).toEqual([
			{ t: 'limit', c: { chat_id: 42, order: 'lt' } },
			{ t: 'limit', c: { chat_id: 42, id: 30, order: 'lt' } },
			{ t: 'limit', c: { chat_id: 42, id: 20, order: 'lt' } }
		]);
	});

	it('stops requesting older messages after the server returns an empty page', async () => {
		request.mockResolvedValue({ list: [] });
		const target = mountPagination();
		await vi.waitFor(() => expect(request).toHaveBeenCalledTimes(1));
		target.dispatchEvent(new Event('scroll'));
		await new Promise<void>((resolve) => queueMicrotask(resolve));

		expect(request).toHaveBeenCalledTimes(1);
		expect(messages.val).toEqual([]);
	});
});

describe('message ordering and mutations', () => {
	it('inserts messages in descending order and replaces duplicate IDs', () => {
		for (const id of [20, 10, 30, 15]) pushUserMessage(id, String(id), []);
		pushUserMessage(20, 'updated', [{ name: 'note.txt', id: 7 }]);

		expect(messages.val.map((message) => message.id)).toEqual([30, 20, 15, 10]);
		expect(messages.val[1].inner).toEqual({
			t: 'user',
			c: { text: 'updated', files: [{ name: 'note.txt', id: 7 }] }
		});
	});

	it('adds a user message only after creation succeeds', async () => {
		const params = {
			chat_id: 42,
			model_id: 3,
			mode: ChatMode.Normal,
			text: 'hello',
			files: [{ name: 'photo.png', id: 7, kind: FileKind.Image }]
		};
		request.mockResolvedValueOnce(undefined).mockResolvedValueOnce({ user_id: 100 });

		await expect(createMessage(params)).resolves.toBe('failed');
		expect(messages.val).toEqual([]);
		await expect(createMessage(params)).resolves.toBe('success');
		expect(messages.val[0]).toMatchObject({
			id: 100,
			inner: { t: 'user', c: { text: params.text, files: params.files } },
			stream: true
		});
		expect(request).toHaveBeenLastCalledWith({
			path: 'message/create',
			body: params,
			token: 'secret'
		});
	});

	it.each([undefined, { deleted: false }])(
		'preserves messages when deletion fails with %j',
		async (response) => {
			for (const id of [30, 20, 10]) pushUserMessage(id, String(id), []);
			request.mockResolvedValue(response);

			await expect(deleteMessage(20)).resolves.toBe('failed');
			expect(messages.val.map((message) => message.id)).toEqual([30, 20, 10]);
		}
	);

	it.each([
		[40, [30, 20, 10]],
		[20, [10]],
		[15, [10]],
		[10, []]
	])('deletes messages at or above %i after server confirmation', async (id, remaining) => {
		for (const messageId of [30, 20, 10]) pushUserMessage(messageId, String(messageId), []);
		request.mockResolvedValue({ deleted: true });

		await expect(deleteMessage(id)).resolves.toBe('success');
		expect(messages.val.map((message) => message.id)).toEqual(remaining);
	});

	it('does not edit a message without a selected chat or model', async () => {
		vi.mocked(getChatId).mockReturnValueOnce(undefined);
		await expect(syncMessage(20, 'edited', [])).resolves.toBe('failed');
		await expect(syncMessage(20, 'edited', [])).resolves.toBe('failed');
		expect(displayError).toHaveBeenCalledWith('internal', 'select model first');
		expect(request).not.toHaveBeenCalled();
	});

	it.each([undefined, { deleted: false }])(
		'does not replace a message when deleting the original fails with %j',
		async (response) => {
			currentRoom.val = { model_id: 3, mode: ChatMode.Normal };
			for (const id of [30, 20, 10]) pushUserMessage(id, String(id), []);
			request.mockResolvedValueOnce(response).mockResolvedValueOnce({ user_id: 40 });

			await expect(syncMessage(20, 'edited', [])).resolves.toBe('failed');
			expect(messages.val.map((message) => message.id)).toEqual([30, 20, 10]);
			expect(request).toHaveBeenCalledTimes(1);
		}
	);

	it('replaces the edited message and removes its later replies after deletion succeeds', async () => {
		currentRoom.val = { model_id: 3, mode: ChatMode.Search };
		for (const id of [30, 20, 10]) pushUserMessage(id, String(id), []);
		request.mockResolvedValueOnce({ deleted: true }).mockResolvedValueOnce({ user_id: 40 });
		const files = [{ id: 7, name: 'note.txt' }];

		await expect(syncMessage(20, 'edited', files)).resolves.toBe('success');
		expect(messages.val.map((message) => message.id)).toEqual([40, 10]);
		expect(request).toHaveBeenNthCalledWith(2, {
			path: 'message/create',
			body: { chat_id: 42, model_id: 3, mode: ChatMode.Search, text: 'edited', files },
			token: 'secret'
		});
	});
});

describe('message SSE state', () => {
	it('merges adjacent text and reasoning while preserving chunk order and identity', async () => {
		pushUserMessage(100, 'question', []);
		await receive([start(), { t: 'reasoning', c: '想' }]);
		const message = messages.val[0];
		const reasoning = chunks()[0];
		await receive([
			{ t: 'reasoning', c: '法' },
			{ t: 'token', c: 'Hello' },
			{ t: 'token', c: ' 世界🙂' },
			{ t: 'reasoning', c: 'more' }
		]);

		expect(messages.val[0]).toBe(message);
		expect(chunks()[0]).toBe(reasoning);
		expect(chunks()).toEqual([
			{ t: 'reasoning', c: '想法' },
			{ t: 'text', c: 'Hello 世界🙂' },
			{ t: 'reasoning', c: 'more' }
		]);
		expect(streaming.val).toBe(true);
	});

	it('resumes text from its UTF-8 byte offset', async () => {
		await receive([start(), { t: 'token', c: '中🙂' }, { t: 'token', c: '文' }]);
		await receive([]);

		expect(streamRequest).toHaveBeenLastCalledWith(
			expect.objectContaining({
				path: 'chat/sse',
				body: { id: 42, resume: { version, cursor: { index: 1, offset: 10 } } },
				token: 'secret'
			})
		);
	});

	it('pairs tool results with calls and advances the cursor across discrete events', async () => {
		const files = [{ id: 7, name: 'result.png', kind: FileKind.Image }];
		const citations = [{ url: 'https://example.com', title: 'Source' }];
		await receive([
			start(),
			{ t: 'tool_call', c: { name: 'search', args: '{"query":"hello"}' } },
			{ t: 'tool_result', c: { content: 'found', files } },
			{ t: 'image', c: 7 },
			{ t: 'url_citation', c: citations },
			{ t: 'title', c: 'A title' }
		]);
		const call = chunks()[0];
		if (call.t !== 'tool_call') throw new Error('Expected a tool call');
		expect(chunks()).toEqual([
			{ t: 'tool_call', c: { id: call.c.id, name: 'search', arg: '{"query":"hello"}' } },
			{ t: 'tool_result', c: { id: call.c.id, response: 'found', files } },
			{ t: 'image', c: 7 },
			{ t: 'url_citation', c: citations }
		]);
		expect(setRoomTitle).toHaveBeenCalledWith(42, 'A title');
		await receive([]);
		expect(streamRequest).toHaveBeenLastCalledWith(
			expect.objectContaining({
				body: { id: 42, resume: { version, cursor: { index: 5, offset: 1 } } }
			})
		);
	});

	it('clears stale messages on version changes and reconnects without a stale cursor', async () => {
		await receive([start(), { t: 'token', c: 'old answer' }]);
		await receive([{ t: 'version', c: ++version }]);

		expect(messages.val).toEqual([]);
		expect(streaming.val).toBe(false);
		await receive([]);
		expect(streamRequest).toHaveBeenLastCalledWith(expect.objectContaining({ body: { id: 42 } }));
	});

	it('preserves messages when the server repeats the current version', async () => {
		await receive([start(), { t: 'token', c: 'answer' }]);
		const message = messages.val[0];
		await receive([{ t: 'version', c: version }]);

		expect(messages.val[0]).toBe(message);
		expect(streaming.val).toBe(true);
	});

	it('finalizes message usage and clears streaming and resume state', async () => {
		pushUserMessage(100, 'question', []);
		await receive([
			start(),
			{ t: 'token', c: 'answer' },
			{ t: 'complete', c: { id: 101, token_count: 25, cost: 0.125, version } }
		]);

		expect(messages.val[0]).toMatchObject({ token_count: 25, price: 0.125, stream: false });
		expect(messages.val[1].stream).toBe(false);
		expect(streaming.val).toBe(false);
		await receive([]);
		expect(streamRequest).toHaveBeenLastCalledWith(expect.objectContaining({ body: { id: 42 } }));
	});

	it('adds stream errors to the active message and tracks their byte length', async () => {
		await receive([start(), { t: 'error', c: '錯誤' }]);
		expect(chunks()).toEqual([{ t: 'error', c: '錯誤' }]);
		await receive([]);
		expect(streamRequest).toHaveBeenLastCalledWith(
			expect.objectContaining({
				body: { id: 42, resume: { version, cursor: { index: 1, offset: 6 } } }
			})
		);
	});

	it('assembles fragmented deep plans and keeps step output separate from the report', async () => {
		const plan: Deep = {
			locale: 'zh-TW',
			has_enough_context: false,
			thought: 'research',
			title: 'Plan',
			steps: [
				{
					need_search: true,
					title: 'Search',
					description: 'Find sources',
					kind: StepKind.Research,
					progress: []
				}
			]
		};
		const serialized = JSON.stringify(plan);
		const warning = vi.spyOn(console, 'warn').mockImplementation(() => {});
		try {
			await receive([
				start(),
				{ t: 'deep_plan', c: serialized.slice(0, 20) },
				{ t: 'deep_plan', c: serialized.slice(20) },
				{ t: 'deep_step_start', c: 0 },
				{ t: 'deep_step_reasoning', c: 'Think' },
				{ t: 'deep_step_reasoning', c: ' more' },
				{ t: 'deep_step_token', c: 'Found' },
				{ t: 'deep_step_token', c: ' sources' },
				{ t: 'deep_report', c: 'Final' },
				{ t: 'deep_report', c: ' report' }
			]);
		} finally {
			warning.mockRestore();
		}

		expect(chunks()).toEqual([
			{
				t: 'deep_agent',
				c: {
					...plan,
					steps: [
						{
							...plan.steps[0],
							progress: [
								{ t: 'reasoning', c: 'Think more' },
								{ t: 'text', c: 'Found sources' }
							]
						}
					]
				}
			},
			{ t: 'text', c: 'Final report' }
		]);
	});

	it('aborts an active request when the document becomes hidden', async () => {
		await receive([start()]);
		const signal = streamRequest.mock.calls.at(-1)?.[0].signal;
		expect(signal?.aborted).toBe(false);
		Object.defineProperty(document, 'visibilityState', { configurable: true, value: 'hidden' });
		document.dispatchEvent(new Event('visibilitychange'));
		expect(signal?.aborted).toBe(true);
	});
});
