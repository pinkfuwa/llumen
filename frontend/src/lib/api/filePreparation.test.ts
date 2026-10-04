import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$lib/image', () => ({ compressImage: vi.fn(), isCompressibleImage: vi.fn() }));
import { compressImage, isCompressibleImage } from '$lib/image';
import { prepareUploadFile } from './filePreparation';

const compress = vi.mocked(compressImage);
const supportsCompression = vi.mocked(isCompressibleImage);
const threshold = 2.5 * 1024 * 1024;

function image(size = threshold + 1) {
	const file = new File(['image'], 'photo.png', { type: 'image/png' });
	Object.defineProperty(file, 'size', { value: size });
	return file;
}

beforeEach(() => {
	supportsCompression.mockResolvedValue(true);
});
afterEach(() => vi.resetAllMocks());

describe('file preparation', () => {
	it.each([threshold - 1, threshold])('keeps an image of %i bytes unchanged', async (size) => {
		const file = image(size);
		await expect(prepareUploadFile(file, new AbortController().signal)).resolves.toBe(file);
		expect(compress).not.toHaveBeenCalled();
	});

	it('keeps unsupported files unchanged regardless of size', async () => {
		const file = image();
		supportsCompression.mockResolvedValue(false);
		await expect(prepareUploadFile(file, new AbortController().signal)).resolves.toBe(file);
		expect(compress).not.toHaveBeenCalled();
	});

	it('compresses supported images above the threshold at quality 0.8', async () => {
		const file = image();
		const prepared = image(100);
		compress.mockResolvedValue(prepared);
		await expect(prepareUploadFile(file, new AbortController().signal)).resolves.toBe(prepared);
		expect(compress).toHaveBeenCalledWith(file, { quality: 0.8 });
	});

	it('falls back to the original file when compression fails', async () => {
		const file = image();
		compress.mockRejectedValue(new Error('canvas failed'));
		await expect(prepareUploadFile(file, new AbortController().signal)).resolves.toBe(file);
	});

	it('does not inspect or compress an already cancelled file', async () => {
		const controller = new AbortController();
		controller.abort();
		await expect(prepareUploadFile(image(), controller.signal)).rejects.toMatchObject({
			name: 'AbortError'
		});
		expect(supportsCompression).not.toHaveBeenCalled();
		expect(compress).not.toHaveBeenCalled();
	});

	it('stops after capability detection if cancelled', async () => {
		const controller = new AbortController();
		let finish!: (supported: boolean) => void;
		supportsCompression.mockReturnValue(
			new Promise((resolve) => {
				finish = resolve;
			})
		);
		const result = prepareUploadFile(image(), controller.signal);
		controller.abort();
		finish(true);
		await expect(result).rejects.toMatchObject({ name: 'AbortError' });
		expect(compress).not.toHaveBeenCalled();
	});

	it.each(['resolve', 'reject'])(
		'does not treat cancellation as compression fallback (%s)',
		async (outcome) => {
			const controller = new AbortController();
			let resolve!: (file: File) => void;
			let reject!: (error: Error) => void;
			compress.mockReturnValue(
				new Promise((resolvePromise, rejectPromise) => {
					resolve = resolvePromise;
					reject = rejectPromise;
				})
			);
			const result = prepareUploadFile(image(), controller.signal);
			await Promise.resolve();
			controller.abort();
			if (outcome === 'resolve') resolve(image(100));
			else reject(new Error('canvas failed'));
			await expect(result).rejects.toMatchObject({ name: 'AbortError' });
		}
	);
});
