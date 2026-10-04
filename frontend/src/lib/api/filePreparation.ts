import { compressImage, isCompressibleImage } from '$lib/image';

const COMPRESS_SIZE_THRESHOLD = 2.5 * 1024 * 1024;

export async function prepareUploadFile(file: File, signal: AbortSignal): Promise<File> {
	if ((await isCompressibleImage(file)) && file.size > COMPRESS_SIZE_THRESHOLD) {
		try {
			const prepared = await compressImage(file, { quality: 0.8 });
			signal.throwIfAborted();
			return prepared;
		} catch {
			return file;
		}
	}
	return file;
}
