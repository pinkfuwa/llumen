import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('./http.svelte', () => ({ RawAPIFetch: vi.fn(), APIFetch: vi.fn() }));
vi.mock('$lib/rune.svelte', () => ({ token: { value: { value: 'secret' } } }));
vi.mock('$lib/error.svelte', () => ({ displayError: vi.fn() }));
vi.mock('$lib/image', () => ({ compressImage: vi.fn(), isCompressibleImage: vi.fn() }));

import { displayError } from '$lib/error.svelte';
import { APIFetch, RawAPIFetch } from './http.svelte';
import { download, downloadCompressed, refresh, upload, uploadFiles } from './files.svelte';

const rawFetchMock = vi.mocked(RawAPIFetch);
const apiFetchMock = vi.mocked(APIFetch);
const objectUrlMock = vi.fn<(blob: Blob | MediaSource) => string>();
const MAX_FILE_SIZE = 100 * 1024 * 1024;

beforeEach(() => {
	objectUrlMock.mockReturnValue('blob:attachment');
	vi.stubGlobal(
		'URL',
		class extends URL {
			static createObjectURL = objectUrlMock;
		}
	);
	vi.stubGlobal('screen', { width: 640 });
	vi.stubGlobal('devicePixelRatio', 1.5);
	vi.spyOn(console, 'warn').mockImplementation(() => {});
});

afterEach(() => {
	vi.restoreAllMocks();
	vi.unstubAllGlobals();
	vi.resetAllMocks();
});

describe('upload', () => {
	it('sends the file, original size, token, and abort signal', async () => {
		const file = new File(['hello'], 'note.txt', { type: 'text/plain' });
		const controller = new AbortController();
		rawFetchMock.mockResolvedValue(new Response('{"id":42}'));

		await expect(upload(file, controller.signal)).resolves.toBe(42);

		const request = rawFetchMock.mock.calls[0][0];
		expect(request).toMatchObject({
			path: 'file/upload',
			token: 'secret',
			signal: controller.signal
		});
		expect(request.body).toBeInstanceOf(FormData);
		const body = request.body as FormData;
		expect(body.get('size')).toBe('5');
		expect(body.get('file')).toBe(file);
		expect(displayError).not.toHaveBeenCalled();
	});

	it('allows a file at the 100MB limit', async () => {
		const file = new File(['hello'], 'limit.txt');
		Object.defineProperty(file, 'size', { value: MAX_FILE_SIZE });
		rawFetchMock.mockResolvedValue(new Response('{"id":42}'));

		await expect(upload(file)).resolves.toBe(42);
		expect(displayError).not.toHaveBeenCalled();
	});

	it('rejects a file above 100MB before sending it', async () => {
		const file = new File(['hello'], 'oversize.txt');
		Object.defineProperty(file, 'size', { value: MAX_FILE_SIZE + 1 });

		await expect(upload(file)).resolves.toBeNull();
		expect(rawFetchMock).not.toHaveBeenCalled();
		expect(displayError).toHaveBeenCalledWith(
			'internal',
			'File size exceeds the maximum limit of 100MB.'
		);
	});

	it.each([undefined, new Response('{}', { status: 413 })])(
		'returns null for an unsuccessful response (%s)',
		async (response) => {
			rawFetchMock.mockResolvedValue(response);

			await expect(upload(new File(['hello'], 'note.txt'))).resolves.toBeNull();
		}
	);
});

describe('uploadFiles', () => {
	it('preserves successful file order and omits failed uploads', async () => {
		const files = ['first.txt', 'failed.txt', 'last.txt'].map((name) => new File(['hello'], name));
		const controller = new AbortController();
		rawFetchMock
			.mockResolvedValueOnce(new Response('{"id":42}'))
			.mockResolvedValueOnce(new Response('{}', { status: 500 }))
			.mockResolvedValueOnce(new Response('{"id":43}'));

		await expect(uploadFiles(files, controller.signal)).resolves.toEqual([
			{ name: 'first.txt', id: 42 },
			{ name: 'last.txt', id: 43 }
		]);
		expect(rawFetchMock).toHaveBeenCalledTimes(3);
		for (const [request] of rawFetchMock.mock.calls) {
			expect(request.signal).toBe(controller.signal);
		}
	});
});

describe('refresh', () => {
	it('skips the request when there are no file IDs', async () => {
		await expect(refresh([])).resolves.toBeNull();
		expect(apiFetchMock).not.toHaveBeenCalled();
	});

	it('returns the renewed expiry for the requested files', async () => {
		apiFetchMock.mockResolvedValue({ valid_until: 1800000000 });

		await expect(refresh([42, 43])).resolves.toBe(1800000000);
		expect(apiFetchMock).toHaveBeenCalledWith({
			path: 'file/refresh',
			body: { ids: [42, 43] },
			token: 'secret'
		});
	});

	it('returns null when refreshing fails', async () => {
		apiFetchMock.mockResolvedValue(undefined);

		await expect(refresh([42])).resolves.toBeNull();
	});
});

describe.each([
	{ name: 'download', read: download, path: 'file/read/42', token: 'secret' },
	{ name: 'downloadCompressed', read: downloadCompressed, path: 'file/image/960/42', token: true }
])('$name', ({ read, path, token }) => {
	it('creates an object URL for the downloaded content', async () => {
		rawFetchMock.mockResolvedValue(
			new Response('image bytes', { headers: { 'Content-Type': 'image/png' } })
		);

		await expect(read(42)).resolves.toBe('blob:attachment');
		expect(rawFetchMock).toHaveBeenCalledWith({ path, method: 'GET', token });
		expect(objectUrlMock).toHaveBeenCalledTimes(1);
		const blob = objectUrlMock.mock.calls[0][0] as Blob;
		expect(blob.type).toBe('image/png');
		await expect(blob.text()).resolves.toBe('image bytes');
	});

	it.each([
		'application/json',
		'application/json; charset=utf-8',
		'Application/JSON; charset=UTF-8'
	])('rejects a JSON error response with content type %s', async (contentType) => {
		rawFetchMock.mockResolvedValue(
			new Response('{"error":"not_found","reason":"expired"}', {
				headers: { 'Content-Type': contentType }
			})
		);

		await expect(read(42)).resolves.toBeUndefined();
		expect(objectUrlMock).not.toHaveBeenCalled();
	});

	it.each([undefined, new Response('not found', { status: 404 })])(
		'does not create an object URL for an unsuccessful response (%s)',
		async (response) => {
			rawFetchMock.mockResolvedValue(response);

			await expect(read(42)).resolves.toBeUndefined();
			expect(objectUrlMock).not.toHaveBeenCalled();
		}
	);
});

it('requests at least 100 pixels for a compressed download', async () => {
	vi.stubGlobal('screen', { width: 10 });
	rawFetchMock.mockResolvedValue(undefined);

	await downloadCompressed(42);

	expect(rawFetchMock).toHaveBeenCalledWith({
		path: 'file/image/100/42',
		method: 'GET',
		token: true
	});
});
