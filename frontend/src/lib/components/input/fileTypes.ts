interface CapabilityFileType {
	image_input: boolean;
	audio_input: boolean;
	other_file_input: boolean;
}

/** Mapping of capability flags to valid MIME‑type lists. */
const IMAGE_TYPES = [
	'image/png',
	'image/jpeg',
	'image/gif',
	'image/webp',
	'image/svg+xml',
	'image/bmp',
	'image/tiff'
];

const AUDIO_TYPES = ['audio/mpeg', 'audio/wav', 'audio/ogg', 'audio/webm', 'audio/flac'];

const OTHER_TYPES = [
	// Documents
	'application/pdf',
	'application/msword',
	'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
	'application/vnd.ms-excel',
	'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
	'application/vnd.ms-powerpoint',
	'application/vnd.openxmlformats-officedocument.presentationml.presentation',
	// Text files
	'text/plain',
	'text/csv',
	'text/markdown'
];

export function getSupportedFileTypes(capability: CapabilityFileType): string {
	const parts: string[] = [];

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
