import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushSync } from 'svelte';

vi.mock('./http.svelte', () => ({ RawAPIFetch: vi.fn(), APIFetch: vi.fn() }));
vi.mock('$lib/rune.svelte', () => ({ token: { value: { value: 'secret' } } }));
vi.mock('$lib/error.svelte', () => ({ displayError: vi.fn() }));
vi.mock('$lib/image', () => ({ compressImage: vi.fn(), isCompressibleImage: vi.fn() }));

import { compressImage, isCompressibleImage } from '$lib/image';
import { RawAPIFetch } from './http.svelte';
import { createUploadPipeline } from './files.svelte';

const request = vi.mocked(RawAPIFetch);
const compress = vi.mocked(compressImage);
const disposers: (() => void)[] = [];

function pipeline(initial: File[] = []) {
	const files = $state({ val: initial });
	let ready!: ReturnType<typeof createUploadPipeline>;
	disposers.push(
		$effect.root(() => {
			ready = createUploadPipeline(() => files.val);
		})
	);
	return { files, ready };
}

beforeEach(() => {
	vi.mocked(isCompressibleImage).mockResolvedValue(false);
	request.mockImplementation(async () => new Response('{"id":42}'));
});

afterEach(() => {
	for (const dispose of disposers.splice(0)) dispose();
	vi.resetAllMocks();
});

describe('createUploadPipeline with Svelte effects', () => {
	it('starts uploads when the selected array is mutated', async () => {
		const { files, ready } = pipeline();
		flushSync();
		files.val.push(new File(['hello'], 'note.txt'));
		flushSync();
		await vi.waitFor(() => expect(request).toHaveBeenCalledTimes(1));
		await expect(ready()).resolves.toEqual([{ name: 'note.txt', id: 42 }]);
	});

	it('includes files selected immediately before ready without flushing effects', async () => {
		const { files, ready } = pipeline();
		flushSync();
		files.val.push(new File(['hello'], 'note.txt'));
		await expect(ready()).resolves.toEqual([{ name: 'note.txt', id: 42 }]);
		expect(request).toHaveBeenCalledTimes(1);
	});

	it('retains completed uploads when another file is added', async () => {
		const { files, ready } = pipeline([new File(['first'], 'first.txt')]);
		flushSync();
		await ready();
		files.val.push(new File(['second'], 'second.txt'));
		flushSync();
		await expect(ready()).resolves.toEqual([
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
		const { ready } = pipeline([original]);
		flushSync();
		await expect(ready()).resolves.toEqual([{ name: prepared.name, id: 42 }]);
		const body = request.mock.calls[0][0].body as FormData;
		expect(body.get('file')).toBe(prepared);
		expect(body.get('size')).toBe(String(prepared.size));
		expect(request.mock.calls[0][0].token).toBe('secret');
	});

	it('settles removal during compression and never uploads the late result', async () => {
		vi.mocked(isCompressibleImage).mockResolvedValue(true);
		let finish!: (file: File) => void;
		compress.mockReturnValue(
			new Promise((resolve) => {
				finish = resolve;
			})
		);
		const original = new File(['image'], 'photo.png');
		Object.defineProperty(original, 'size', { value: 3 * 1024 * 1024 });
		const { files, ready } = pipeline([original]);
		flushSync();
		await vi.waitFor(() => expect(compress).toHaveBeenCalledTimes(1));
		const result = ready();
		files.val = [];
		flushSync();
		await expect(result).resolves.toEqual([]);
		finish(new File(['compressed'], 'photo.png'));
		await new Promise((resolve) => setTimeout(resolve, 0));
		expect(request).not.toHaveBeenCalled();
	});

	it('aborts requests and settles ready when the effect scope is destroyed', async () => {
		let finish!: (response: Response) => void;
		request.mockReturnValue(
			new Promise((resolve) => {
				finish = resolve;
			})
		);
		const { ready } = pipeline([new File(['hello'], 'note.txt')]);
		flushSync();
		await vi.waitFor(() => expect(request).toHaveBeenCalledTimes(1));
		const result = ready();
		disposers.pop()!();
		expect(request.mock.calls[0][0].signal?.aborted).toBe(true);
		await expect(result).resolves.toEqual([]);
		finish(new Response('{"id":42}'));
		await new Promise((resolve) => setTimeout(resolve, 0));
	});
});
