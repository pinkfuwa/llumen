import { RawAPIFetch, APIFetch } from './state/errorHandle';
import type { FileUploadResp, FileRefreshReq, FileRefreshResp } from './types';
import { dispatchError } from '$lib/error';
import { untrack } from 'svelte';

const MAX_FILE_SIZE = 100 * 1024 * 1024; // backend limit body size at 128 MB, body size!=file size

export async function upload(file: File, signal?: AbortSignal): Promise<number | null> {
	const formData = new FormData();
	if (file.size > MAX_FILE_SIZE) {
		dispatchError('internal', 'File size exceeds the maximum limit of 100MB.');
		return null;
	}

	formData.append('size', file.size.toString());
	formData.append('file', file);

	const response = await RawAPIFetch('file/upload', formData, 'POST', signal);

	if (!response.ok) {
		console.warn('Fail to upload', { file });
		return null;
	}

	const data = (await response.json()) as FileUploadResp;
	return data.id;
}

export async function refresh(fileIds: number[]): Promise<number | null> {
	if (fileIds.length === 0) return null;

	const response = await APIFetch<FileRefreshResp, FileRefreshReq>('file/refresh', {
		ids: fileIds
	});

	return response?.valid_until ?? null;
}

export async function download(id: number): Promise<string | undefined> {
	const response = await RawAPIFetch<undefined>(
		`file/read/${encodeURIComponent(id)}`,
		undefined,
		'GET'
	);

	let content_type = response.headers.get('Content-Type');
	if (!response.ok || content_type == 'application/json') {
		console.warn('Fail to download', { id });
		return;
	}

	const blob = await response.blob();
	return URL.createObjectURL(blob);
}

export async function downloadCompressed(id: number): Promise<string | undefined> {
	const width = Math.max(Math.ceil(window.devicePixelRatio * screen.width), 100);
	const response = await RawAPIFetch<undefined>(
		`file/image/${width}/${encodeURIComponent(id)}`,
		undefined,
		'GET'
	);

	let content_type = response.headers.get('Content-Type');
	if (!response.ok || content_type == 'application/json') {
		console.warn('Fail to download compressed image', { id, width });
		return;
	}

	const blob = await response.blob();
	return URL.createObjectURL(blob);
}

/**
 * Upload multiple files and return their metadata
 */
export async function uploadFiles(
	files: File[],
	signal?: AbortSignal
): Promise<{ name: string; id: number }[]> {
	const results: { name: string; id: number }[] = [];

	for (const file of files) {
		const id = await upload(file, signal);
		if (id !== null) {
			results.push({ name: file.name, id });
		}
	}

	return results;
}

/**
 * Create an upload effect that manages file uploads reactively.
 * Returns an async function that ensures all uploads are complete before resolving.
 *
 * @param fileGetter - Function that returns the current list of files to upload
 * @returns An async function that uploads all files and returns their metadata
 *
 * @example
 * ```ts
 * let files = $state<File[]>([]);
 * const ensureUploaded = createUploadEffect(() => files);
 *
 * async function submit() {
 *   const uploadedFiles = await ensureUploaded();
 *   // use uploadedFiles...
 * }
 * ```
 */
export function createUploadEffect(
	fileGetter: () => File[]
): () => Promise<{ name: string; id: number }[]> {
	let uploads = $state<Map<string, { promise: Promise<number | null>; file: File }>>(new Map());
	let abortController = new AbortController();

	// Track which files need to be uploaded
	$effect(() => {
		const currentFiles = fileGetter();

		// Create a set of current file keys
		const currentKeys = new Set(
			currentFiles.map((f) => {
				try {
					return `${f.name}-${f.size}`;
				} catch {
					return '';
				}
			})
		);

		// Remove uploads that are no longer in the file list
		// Use untrack to avoid creating a dependency on uploads (prevent infinite loop)
		const newUploads = new Map(untrack(() => uploads));
		for (const [key] of untrack(() => uploads)) {
			if (!currentKeys.has(key)) {
				newUploads.delete(key);
			}
		}

		// Add new uploads
		for (const file of currentFiles) {
			let key: string;
			try {
				key = `${file.name}-${file.size}`;
			} catch {
				continue;
			}

			if (!newUploads.has(key)) {
				newUploads.set(key, {
					promise: upload(file, abortController.signal),
					file
				});
			}
		}

		uploads = newUploads;
	});

	// Cleanup on component unmount
	$effect(() => {
		return () => {
			abortController.abort();
		};
	});

	return async function ensureUploaded(): Promise<{ name: string; id: number }[]> {
		const currentFiles = untrack(() => fileGetter());
		const results: { name: string; id: number }[] = [];

		for (const file of currentFiles) {
			let key: string;
			try {
				key = `${file.name}-${file.size}`;
			} catch {
				continue;
			}

			const upload = untrack(() => uploads.get(key));
			if (upload) {
				const id = await upload.promise;
				if (id !== null) {
					results.push({ name: file.name, id });
				}
			}
		}

		return results;
	};
}
