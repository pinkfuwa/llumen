import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushSync } from 'svelte';

vi.mock('./http.svelte', () => ({ RawAPIFetch: vi.fn(), APIFetch: vi.fn() }));
vi.mock('$lib/rune.svelte', () => ({ token: { value: { value: 'secret' } } }));
vi.mock('$lib/error.svelte', () => ({ displayError: vi.fn() }));
vi.mock('$lib/image', () => ({ compressImage: vi.fn(), isCompressibleImage: vi.fn() }));

import { compressImage, isCompressibleImage } from '$lib/image';
import { RawAPIFetch } from './http.svelte';
import { watchSelectedFileUploads } from './files.svelte';

const request = vi.mocked(RawAPIFetch);
const compress = vi.mocked(compressImage);
const destroyScopes: (() => void)[] = [];

function watchFiles(initial: File[] = []) {
	const files = $state({ val: initial });
	let waitForUploads!: ReturnType<typeof watchSelectedFileUploads>;
	destroyScopes.push(
		$effect.root(() => {
			waitForUploads = watchSelectedFileUploads(() => files.val);
		})
	);
	return { files, waitForUploads };
}

beforeEach(() => {
	vi.mocked(isCompressibleImage).mockResolvedValue(false);
	request.mockImplementation(async () => new Response('{"id":42}'));
});

afterEach(() => {
	for (const destroyScope of destroyScopes.splice(0)) destroyScope();
	vi.resetAllMocks();
});

describe('watchSelectedFileUploads with Svelte effects', () => {
	it('starts uploads when the selected array is mutated', async () => {
		const { files, waitForUploads } = watchFiles();
		flushSync();
		files.val.push(new File(['hello'], 'note.txt'));
		flushSync();
		await vi.waitFor(() => expect(request).toHaveBeenCalledTimes(1));
		await expect(waitForUploads()).resolves.toEqual([{ name: 'note.txt', id: 42 }]);
	});

	it('includes files selected immediately before waitForUploads without flushing effects', async () => {
		const { files, waitForUploads } = watchFiles();
		flushSync();
		files.val.push(new File(['hello'], 'note.txt'));
		await expect(waitForUploads()).resolves.toEqual([{ name: 'note.txt', id: 42 }]);
		expect(request).toHaveBeenCalledTimes(1);
	});

	it('retains completed uploads when another file is added', async () => {
		const { files, waitForUploads } = watchFiles([new File(['first'], 'first.txt')]);
		flushSync();
		await waitForUploads();
		files.val.push(new File(['second'], 'second.txt'));
		flushSync();
		await expect(waitForUploads()).resolves.toEqual([
			{ name: 'first.txt', id: 42 },
			{ name: 'second.txt', id: 42 }
		]);
		expect(request).toHaveBeenCalledTimes(2);
	});

	it('uploads the prepared file with its name and compressed size', async () => {
		vi.mocked(isCompressibleImage).mockResolvedValue(true);
		const original = new File(['image'], 'photo.png');
		Object.defineProperty(original, 'size', { value: 3 * 1024 * 1024 });
		const prepared = new File(['compressed'], 'prepared.png');
		compress.mockResolvedValue(prepared);
		const { waitForUploads } = watchFiles([original]);
		flushSync();
		await expect(waitForUploads()).resolves.toEqual([{ name: prepared.name, id: 42 }]);
		const body = request.mock.calls[0][0].body as FormData;
		expect(body.get('file')).toBe(prepared);
		expect(body.get('size')).toBe(String(prepared.size));
		expect(request.mock.calls[0][0].token).toBe('secret');
	});

	it('returns no files after removal during compression and does not upload the compressed file', async () => {
		vi.mocked(isCompressibleImage).mockResolvedValue(true);
		let finish!: (file: File) => void;
		compress.mockReturnValue(
			new Promise((resolve) => {
				finish = resolve;
			})
		);
		const original = new File(['image'], 'photo.png');
		Object.defineProperty(original, 'size', { value: 3 * 1024 * 1024 });
		const { files, waitForUploads } = watchFiles([original]);
		flushSync();
		await vi.waitFor(() => expect(compress).toHaveBeenCalledTimes(1));
		const result = waitForUploads();
		files.val = [];
		flushSync();
		await expect(result).resolves.toEqual([]);
		finish(new File(['compressed'], 'photo.png'));
		await new Promise((resolve) => setTimeout(resolve, 0));
		expect(request).not.toHaveBeenCalled();
	});

	it('aborts requests and returns no files when the effect scope is destroyed', async () => {
		let finish!: (response: Response) => void;
		request.mockReturnValue(
			new Promise((resolve) => {
				finish = resolve;
			})
		);
		const { waitForUploads } = watchFiles([new File(['hello'], 'note.txt')]);
		flushSync();
		await vi.waitFor(() => expect(request).toHaveBeenCalledTimes(1));
		const result = waitForUploads();
		destroyScopes.pop()!();
		expect(request.mock.calls[0][0].signal?.aborted).toBe(true);
		await expect(result).resolves.toEqual([]);
		finish(new Response('{"id":42}'));
		await new Promise((resolve) => setTimeout(resolve, 0));
	});
});
