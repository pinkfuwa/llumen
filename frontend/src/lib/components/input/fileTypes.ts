interface CapabilityFileType {
	image_input: boolean;
	audio_input: boolean;
	other_file_input: boolean;
}

/** Mapping of capability flags to valid MIME‑type lists. */
const IMAGE_TYPES = ['avif', 'webp', 'bmp', 'gif', 'png', 'jpg', 'jpeg'];

const AUDIO_TYPES = ['audio/*'];

const OTHER_TYPES = ['pdf', 'doc', 'ppt', 'pptx', 'docs', 'xlsx'];

// file that will directly upload without any processing
const LITERAL_FILE = [
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

export function getSupportedFileExtensions(capability?: CapabilityFileType): string[] {
	if (!capability) {
		return [];
	}

	const parts: string[] = [];

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

	return parts;
}

export function getAllFileExtensions(): string[] {
	return [...LITERAL_FILE, ...IMAGE_TYPES, ...AUDIO_TYPES, ...OTHER_TYPES];
}
