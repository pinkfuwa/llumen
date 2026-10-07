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
import { ChatMode, FileKind } from './types';
import type { SseResp } from './types';

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
