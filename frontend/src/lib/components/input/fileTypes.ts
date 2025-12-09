interface CapabilityFileType {
	image_input: boolean;
	audio_input: boolean;
	other_file_input: boolean;
}

/** Mapping of capability flags to valid MIME‑type lists. */
const IMAGE_TYPES = [
	'image/avcs',
	'image/avif',
	'image/bmp',
	'image/gif',
	'image/png',
	'image/webp',
	'image/jpeg',
	'image/webp',
	'image/apng',
	'image/bmp',
	'image/vnd.wap.wbmp'
];

const AUDIO_TYPES = ['audio/*'];

const OTHER_TYPES = [
	'application/pdf',
	'application/msword',
	'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
	'application/vnd.ms-excel',
	'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
	'application/vnd.ms-powerpoint',
	'application/vnd.openxmlformats-officedocument.presentationml.presentation'
];

// file that will directly upload without any processing
const LITERAL_FILE = [
	'text/*',
	'image/svg+xml',
	'.md',
	'.txt',
	'.ts',
	'.rs',
	'.py',
	'.svelte',
	'.svelte.ts',
	'.json',
	'.csv',
	'.c',
	'.cpp',
	'.h',
	'.hpp',
	'.toml'
];

export function getSupportedFileTypes(capability: CapabilityFileType): string {
	const parts: string[] = [];

	// clone the array to avoid mutation
	parts.push(...LITERAL_FILE);

	if (capability.image_input) {
		parts.push(...IMAGE_TYPES);
	}

	if (capability.audio_input) {
		parts.push(...AUDIO_TYPES);
	}

	if (capability.other_file_input) {
		parts.push(...OTHER_TYPES);
	}

	return parts.join(',');
}
