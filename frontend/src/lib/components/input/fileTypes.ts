interface CapabilityFileType {
	image_input: boolean;
	audio_input: boolean;
	native_file_input: boolean;
	ocr_file_input: boolean;
}

/** Mapping of capability flags to valid MIME‑type lists. */
const IMAGE_TYPES = ['avif', 'webp', 'bmp', 'gif', 'png', 'jpg', 'jpeg'];

const AUDIO_TYPES = ['audio/*'];

const NATIVE_TYPES: string[] = ['pdf'];

// file that will directly upload without any processing
const LITERAL_TYPES = [
	'md',
	'txt',
	'ts',
	'rs',
	'py',
	'svelte',
	'json',
	'csv',
	'c',
	'cpp',
	'h',
	'hpp',
	'toml',
	'text',
	'js'
];

const OCR_TYPES = ['pdf'];

export function getSupportedFileExtensions(capability?: CapabilityFileType): string[] {
	if (!capability) {
		return [];
	}

	const parts: string[] = [];

	parts.push(...LITERAL_TYPES);

	if (capability.image_input) {
		parts.push(...IMAGE_TYPES);
	}

	if (capability.audio_input) {
		parts.push(...AUDIO_TYPES);
	}

	if (capability.native_file_input) {
		parts.push(...NATIVE_TYPES);
	}

	if (capability.ocr_file_input) {
		parts.push(...OCR_TYPES);
	}

	return parts;
}

export function getAllFileExtensions(): string[] {
	return [...LITERAL_TYPES, ...IMAGE_TYPES, ...AUDIO_TYPES, ...NATIVE_TYPES];
}
