import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$lib/error.svelte', () => ({ displayError: vi.fn() }));
vi.mock('$lib/rune.svelte', () => ({ token: { value: undefined } }));

import { displayError } from '$lib/error.svelte';
import { APIFetch, RawAPIFetch } from './http.svelte';

const fetchMock = vi.fn<typeof fetch>();

beforeEach(() => {
	vi.stubGlobal('fetch', fetchMock);
});

afterEach(() => {
	vi.useRealTimers();
	vi.unstubAllGlobals();
	vi.clearAllMocks();
	fetchMock.mockReset();
});

describe('RawAPIFetch', () => {
	it('sends JSON with an explicit token to the API path', async () => {
		const response = new Response('{}');
		fetchMock.mockResolvedValue(response);

		expect(
			await RawAPIFetch({ path: 'message/create', body: { text: 'hello' }, token: 'secret' })
		).toBe(response);
		expect(fetchMock).toHaveBeenCalledWith('/api/message/create', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json', Authorization: 'secret' },
			body: '{"text":"hello"}',
			signal: undefined
		});
	});

	it('sends FormData without a JSON content type', async () => {
		fetchMock.mockResolvedValue(new Response('{}'));
		const body = new FormData();
		body.set('file', new Blob(['hello']), 'note.txt');

		await RawAPIFetch({ path: 'file/upload', body, token: false });

		expect(fetchMock).toHaveBeenCalledWith('/api/file/upload', {
			method: 'POST',
			headers: {},
			body,
			signal: undefined
		});
	});

	it('rejects a path with a leading slash before fetching', () => {
		expect(() => RawAPIFetch({ path: '/message/create', token: false })).toThrow('Invalid path');
		expect(fetchMock).not.toHaveBeenCalled();
	});

	it('retries network failures with exponential backoff when requested', async () => {
		vi.useFakeTimers();
		const response = new Response('{}');
		fetchMock
			.mockRejectedValueOnce(new TypeError('offline'))
			.mockRejectedValueOnce(new TypeError('offline'))
			.mockResolvedValueOnce(response);

		const result = RawAPIFetch({ path: 'message/paginate', retry: true, token: false });
		expect(fetchMock).toHaveBeenCalledTimes(1);
		await vi.advanceTimersByTimeAsync(199);
		expect(fetchMock).toHaveBeenCalledTimes(1);
		await vi.advanceTimersByTimeAsync(1);
		expect(fetchMock).toHaveBeenCalledTimes(2);
		await vi.advanceTimersByTimeAsync(400);
		expect(fetchMock).toHaveBeenCalledTimes(3);
		expect(await result).toBe(response);
	});

	it('does not retry an HTTP response', async () => {
		const response = new Response('{}', { status: 503 });
		fetchMock.mockResolvedValue(response);

		expect(await RawAPIFetch({ path: 'message/paginate', retry: true, token: false })).toBe(
			response
		);
		expect(fetchMock).toHaveBeenCalledTimes(1);
	});

	it('does not retry a network failure by default', async () => {
		fetchMock.mockRejectedValue(new TypeError('offline'));

		await expect(RawAPIFetch({ path: 'message/create', token: false })).rejects.toThrow('offline');
		expect(fetchMock).toHaveBeenCalledTimes(1);
	});

	it('stops after three retries', async () => {
		vi.useFakeTimers();
		fetchMock.mockRejectedValue(new TypeError('offline'));

		const result = RawAPIFetch({ path: 'message/paginate', retry: true, token: false });
		const rejection = expect(result).rejects.toThrow('offline');
		await vi.advanceTimersByTimeAsync(1400);

		await rejection;
		expect(fetchMock).toHaveBeenCalledTimes(4);
	});

	it('stops retrying when aborted during backoff', async () => {
		vi.useFakeTimers();
		fetchMock.mockRejectedValue(new TypeError('offline'));
		const controller = new AbortController();
		const result = RawAPIFetch({
			path: 'message/paginate',
			retry: true,
			token: false,
			signal: controller.signal
		});
		await vi.advanceTimersByTimeAsync(0);
		controller.abort();

		await expect(result).rejects.toMatchObject({ name: 'AbortError' });
		expect(fetchMock).toHaveBeenCalledTimes(1);
	});
});

describe('APIFetch', () => {
	it('returns successful JSON', async () => {
		fetchMock.mockResolvedValue(new Response('{"id":42}'));

		await expect(APIFetch<{ id: number }>({ path: 'message/get', token: false })).resolves.toEqual({
			id: 42
		});
		expect(displayError).not.toHaveBeenCalled();
	});

	it('displays a structured API error', async () => {
		fetchMock.mockResolvedValue(new Response('{"error":"invalid","reason":"bad input"}'));

		await expect(APIFetch({ path: 'message/get', token: false })).resolves.toBeUndefined();
		expect(displayError).toHaveBeenCalledWith('invalid', 'bad input');
	});

	it('displays a generic error for a network failure', async () => {
		fetchMock.mockRejectedValue(new TypeError('offline'));

		await expect(APIFetch({ path: 'message/get', token: false })).resolves.toBeUndefined();
		expect(displayError).toHaveBeenCalledWith('API(typeshare)', 'maybe backend is disconnected');
	});
});
