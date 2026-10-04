import { compressImage, isCompressibleImage } from '$lib/image';

const COMPRESS_SIZE_THRESHOLD = 2.5 * 1024 * 1024;

export async function prepareUploadFile(file: File, signal: AbortSignal): Promise<File> {
	signal.throwIfAborted();
	const compressible = await isCompressibleImage(file);
	signal.throwIfAborted();
	if (!compressible || file.size <= COMPRESS_SIZE_THRESHOLD) return file;

	let prepared: File;
	try {
		prepared = await compressImage(file, { quality: 0.8 });
	} catch {
		signal.throwIfAborted();
		return file;
	}
	signal.throwIfAborted();
	return prepared;
}
