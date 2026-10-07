import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushSync } from 'svelte';

vi.mock('$app/state', () => ({
	page: { params: { id: '42' }, route: { id: '/chat/[id]' } }
}));
vi.mock('$app/navigation', () => ({ goto: vi.fn(async () => {}) }));
vi.mock('./http.svelte', () => ({ APIFetch: vi.fn(async () => undefined) }));
vi.mock('$lib/rune.svelte', () => ({ token: { value: { value: 'secret' } } }));

import { page } from '$app/state';
import { goto } from '$app/navigation';
import { token } from '$lib/rune.svelte';
import { APIFetch } from './http.svelte';
import {
	chatrooms,
	createRoom,
	currentRoom,
	deleteEntry,
	getChatId,
	haltCompletion,
	setRoomTitle,
	syncEntry
} from './chatroom.svelte';
import { ChatMode, FileKind } from './types';

const request = vi.mocked(APIFetch);
const visibilityListeners: EventListenerOrEventListenerObject[] = [];
const room = { model_id: 7, mode: ChatMode.Normal, title: 'Original', owner_id: 1 };
const creation = {
	message: 'Explain this diagram',
	modelId: 7,
	files: [{ id: 9, name: 'diagram.png', kind: FileKind.Image }],
	mode: ChatMode.Search
};

beforeAll(async () => {
	const addListener = vi.spyOn(document, 'addEventListener');
	flushSync();
	for (const [event, listener] of addListener.mock.calls) {
		if (event === 'visibilitychange') visibilityListeners.push(listener);
	}
	addListener.mockRestore();
	await Promise.resolve();
});

afterAll(() => {
	for (const listener of visibilityListeners) {
		document.removeEventListener('visibilitychange', listener);
	}
});

beforeEach(() => {
	vi.clearAllMocks();
	request.mockReset().mockResolvedValue(undefined);
	page.params.id = '42';
	token.value = { value: 'secret', expireAt: '', renewAt: '' };
	chatrooms.val = [
		{ id: 100, name: 'Newest' },
		{ id: 42, name: 'Original' },
		{ id: 10, name: 'Oldest' }
	];
	currentRoom.val = { ...room };
});

describe('chat list mutations', () => {
	it.each([100, 42, 10])('deletes chat %i without removing neighboring entries', async (id) => {
		request.mockResolvedValueOnce({ deleted: true });

		await expect(deleteEntry(id)).resolves.toBe('success');

		expect(request).toHaveBeenCalledWith({
			path: 'chat/delete',
			body: { id },
			token: 'secret'
		});
		expect(chatrooms.val.map((entry) => entry.id)).toEqual(
			[100, 42, 10].filter((entryId) => entryId !== id)
		);
	});

	it.each([101, 50, 1])('leaves the list intact when deleted chat %i is absent', async (id) => {
		request.mockResolvedValueOnce({ deleted: true });
		await expect(deleteEntry(id)).resolves.toBe('success');
		expect(chatrooms.val.map((entry) => entry.id)).toEqual([100, 42, 10]);
	});

	it.each([undefined, { deleted: false }])(
		'keeps the chat when deletion fails (%s)',
		async (response) => {
			request.mockResolvedValueOnce(response);
			await expect(deleteEntry(42)).resolves.toBe('failed');
			expect(chatrooms.val.map((entry) => entry.id)).toEqual([100, 42, 10]);
		}
	);

	it('updates the active title while preserving room metadata', () => {
		setRoomTitle(42, 'Renamed');
		expect(chatrooms.val[1]).toEqual({ id: 42, name: 'Renamed' });
		expect(currentRoom.val).toEqual({ ...room, title: 'Renamed' });
	});

	it('updates another chat without changing the active room', () => {
		setRoomTitle(10, 'Older chat');
		expect(chatrooms.val[2]).toEqual({ id: 10, name: 'Older chat' });
		expect(currentRoom.val).toEqual(room);
	});

	it('updates the active room even when its sidebar entry is absent', () => {
		chatrooms.val = [];
		setRoomTitle(42, 'Renamed');
		expect(chatrooms.val).toEqual([]);
		expect(currentRoom.val?.title).toBe('Renamed');
	});

	it('applies a title only after the server confirms the write', async () => {
		request.mockResolvedValueOnce({ wrote: true });
		await expect(syncEntry(42, 'Renamed')).resolves.toBe('success');
		expect(request).toHaveBeenCalledWith({
			path: 'chat/write',
			body: { chat_id: 42, title: 'Renamed' },
			token: 'secret'
		});
		expect(currentRoom.val?.title).toBe('Renamed');
		expect(chatrooms.val[1].name).toBe('Renamed');
	});

	it.each([undefined, { wrote: false }])(
		'preserves titles after a failed write (%s)',
		async (response) => {
			request.mockResolvedValueOnce(response);
			await expect(syncEntry(42, 'Renamed')).resolves.toBe('failed');
			expect(currentRoom.val).toEqual(room);
			expect(chatrooms.val[1].name).toBe('Original');
		}
	);
});

describe('createRoom', () => {
	it('creates the chat and first message before adding the entry and navigating', async () => {
		request.mockResolvedValueOnce({ id: 101 });
		request.mockImplementationOnce(async () => {
			expect(chatrooms.val.map((entry) => entry.id)).toEqual([100, 42, 10]);
			expect(goto).not.toHaveBeenCalled();
			return { id: 500 };
		});

		await expect(createRoom(creation)).resolves.toBe('success');

		expect(request).toHaveBeenNthCalledWith(1, {
			path: 'chat/create',
			body: { model_id: 7, mode: ChatMode.Search },
			token: 'secret'
		});
		expect(request).toHaveBeenNthCalledWith(2, {
			path: 'message/create',
			body: {
				chat_id: 101,
				model_id: 7,
				mode: ChatMode.Search,
				text: creation.message,
				files: creation.files
			},
			token: 'secret'
		});
		expect(chatrooms.val[0]).toEqual({ id: 101, name: creation.message });
		expect(goto).toHaveBeenCalledExactlyOnceWith('/chat/101');
	});

	it('does not create a chat without a token', async () => {
		token.value = undefined;
		await expect(createRoom(creation)).resolves.toBe('failed');
		expect(request).not.toHaveBeenCalled();
		expect(goto).not.toHaveBeenCalled();
	});

	it('does not submit the message when chat creation fails', async () => {
		await expect(createRoom(creation)).resolves.toBe('failed');
		expect(request).toHaveBeenCalledTimes(1);
		expect(chatrooms.val.map((entry) => entry.id)).toEqual([100, 42, 10]);
		expect(goto).not.toHaveBeenCalled();
	});

	it('keeps the list and route intact when first-message creation fails', async () => {
		request.mockResolvedValueOnce({ id: 101 });
		await expect(createRoom(creation)).resolves.toBe('failed');
		expect(request).toHaveBeenCalledTimes(2);
		expect(chatrooms.val.map((entry) => entry.id)).toEqual([100, 42, 10]);
		expect(goto).not.toHaveBeenCalled();
	});
});

it.each(['new', 'invalid', undefined])('does not treat route id %s as a chat id', (id) => {
	page.params.id = id;
	expect(getChatId()).toBeUndefined();
});

it('reads numeric chat ids and uses the explicit token to halt a completion', async () => {
	expect(getChatId()).toBe(42);
	await haltCompletion({ id: 42 });
	expect(request).toHaveBeenCalledExactlyOnceWith({
		path: 'chat/halt',
		body: { id: 42 },
		token: 'secret'
	});
});
