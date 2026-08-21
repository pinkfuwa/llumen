export interface CapabilityFileType {
	image_input: boolean;
	audio_input: boolean;
	video_input: boolean;
	native_file_input: boolean;
	ocr_file_input: boolean;
}

export interface SupportedFileTypes {
	mimes: string[];
	extensions: string[];
}

const IMAGE_EXTENSIONS = [
	'avif',
	'webp',
	'bmp',
	'gif',
	'png',
	'jpg',
	'jpeg',
	'svg',
	'tiff',
	'tif',
	'ico',
	'heic',
	'heif'
];

const AUDIO_EXTENSIONS = ['m4a', 'ogg'];

const VIDEO_EXTENSIONS = ['mp4', 'mpeg', 'mov', 'webm'];

const PDF_EXTENSIONS = ['pdf'];

export function getSupportedFileTypes(capability?: CapabilityFileType): SupportedFileTypes {
	if (!capability) {
		return { mimes: [], extensions: [] };
	}

	const mimes: string[] = [];
	const extensions: string[] = [];

	if (capability.image_input) {
		mimes.push('image/*');
		extensions.push(...IMAGE_EXTENSIONS);
	}

	if (capability.audio_input) {
		mimes.push('audio/*');
		extensions.push(...AUDIO_EXTENSIONS);
	}

	if (capability.video_input) {
		mimes.push('video/*');
		extensions.push(...VIDEO_EXTENSIONS);
	}

	if (capability.native_file_input || capability.ocr_file_input) {
		mimes.push('application/pdf');
		extensions.push(...PDF_EXTENSIONS);
	}

	return { mimes, extensions };
}

function normalizeMime(mime: string): string {
	return mime.split(';')[0]?.trim().toLowerCase() ?? '';
}

export function isMimeSupported(mime: string, mimes: string[]): boolean {
	if (!mime || mimes.length === 0) return false;

	const normalized = normalizeMime(mime);
	if (!normalized) return false;

	return mimes.some((matcher) => {
		const candidate = matcher.toLowerCase();
		if (candidate.endsWith('/*')) {
			return normalized.startsWith(candidate.slice(0, -1));
		}
		return normalized === candidate;
	});
}

function fileExtension(name: string): string {
	const base = name.split(/[/\\]/).pop() ?? name;
	const dot = base.lastIndexOf('.');
	if (dot <= 0 || dot === base.length - 1) return '';
	return base.slice(dot + 1).toLowerCase();
}

export function isExtensionSupported(name: string, extensions: string[]): boolean {
	if (extensions.length === 0) return false;
	const ext = fileExtension(name);
	return ext !== '' && extensions.includes(ext);
}

export function isFileTypeSupported(
	file: { name: string; type?: string },
	supported: SupportedFileTypes
): boolean {
	if (file.type && isMimeSupported(file.type, supported.mimes)) return true;
	return isExtensionSupported(file.name, supported.extensions);
}

async function* fileChunks(file: File): AsyncGenerator<Uint8Array> {
	if (typeof file.stream === 'function') {
		const reader = file.stream().getReader();
		try {
			while (true) {
				const { done, value } = await reader.read();
				if (done) return;
				if (value && value.byteLength > 0) yield value;
			}
		} finally {
			reader.releaseLock();
		}
		return;
	}

	const buffer = new Uint8Array(await file.arrayBuffer());
	if (buffer.byteLength > 0) yield buffer;
}

function looksLikeUtf16Le(bytes: Uint8Array): boolean {
	return bytes.length >= 2 && bytes[0] === 0xff && bytes[1] === 0xfe;
}

function looksLikeUtf16Be(bytes: Uint8Array): boolean {
	return bytes.length >= 2 && bytes[0] === 0xfe && bytes[1] === 0xff;
}

export async function isTextFile(file: File): Promise<boolean> {
	if (file.size === 0) return true;

	let utf16: TextDecoder | null = null;
	const utf8 = new TextDecoder('utf-8', { fatal: true });
	let first = true;

	try {
		for await (const chunk of fileChunks(file)) {
			if (first) {
				first = false;
				if (looksLikeUtf16Le(chunk)) {
					utf16 = new TextDecoder('utf-16le', { fatal: true });
				} else if (looksLikeUtf16Be(chunk)) {
					utf16 = new TextDecoder('utf-16be', { fatal: true });
				}
			}

			if (utf16) {
				utf16.decode(chunk, { stream: true });
			} else {
				if (chunk.includes(0)) return false;
				utf8.decode(chunk, { stream: true });
			}
		}

		if (utf16) {
			utf16.decode();
		} else {
			utf8.decode();
		}
		return true;
	} catch {
		return false;
	}
}

export async function separateFiles(
	files: File[],
	supported: SupportedFileTypes
): Promise<{
	supported: File[];
	unsupported: File[];
}> {
	const accepted: File[] = [];
	const leftovers: File[] = [];

	for (const file of files) {
		if (isFileTypeSupported(file, supported)) {
			accepted.push(file);
		} else {
			leftovers.push(file);
		}
	}

	const unsupported: File[] = [];
	for (const file of leftovers) {
		if (await isTextFile(file)) {
			accepted.push(file);
		} else {
			unsupported.push(file);
		}
	}

	return { supported: accepted, unsupported };
}
