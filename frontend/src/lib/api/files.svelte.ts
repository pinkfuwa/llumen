import { RawAPIFetch, APIFetch } from './state/errorHandle';
import type { FileUploadResp, FileRefreshReq, FileRefreshResp } from './types';
import { compressImage, isImageFile } from '$lib/image';
import { dispatchError } from '$lib/error';
import { untrack } from 'svelte';

const MAX_FILE_SIZE = 100 * 1024 * 1024;
const COMPRESS_SIZE_THRESHOLD = 2.5 * 1024 * 1024;

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

function fileKey(file: File): string {
	return `${file.name}-${file.size}`;
}

interface PendingEntry {
	prepare: Promise<File>;
	upload: Promise<number | null>;
	controller: AbortController;
	file: File;
}

export function createUploadPipeline(
	fileGetter: () => File[]
): () => Promise<{ name: string; id: number }[]> {
	let pending = $state<Map<string, PendingEntry>>(new Map());

	$effect(() => {
		const currentFiles = fileGetter();

		const currentKeys = new Set(currentFiles.map(fileKey));

		const next = new Map(untrack(() => pending));
		for (const [key, entry] of untrack(() => pending)) {
			if (!currentKeys.has(key)) {
				next.delete(key);
				entry.controller.abort();
			}
		}

		for (const file of currentFiles) {
			const key = fileKey(file);
			if (next.has(key)) continue;

			const controller = new AbortController();

			const prepare: Promise<File> = isImageFile(file) && file.size > COMPRESS_SIZE_THRESHOLD
				? compressImage(file, { quality: 0.8 }).catch(() => file).then((f) => {
						controller.signal.throwIfAborted();
						return f;
					})
				: Promise.resolve(file);

			const uploadP = prepare.then((f) => upload(f, controller.signal));

			next.set(key, { prepare, upload: uploadP, controller, file });
		}

		pending = next;
	});

	$effect(() => {
		return () => {
			for (const [, entry] of untrack(() => pending)) {
				entry.controller.abort();
			}
		};
	});

	return async function ensureReady(): Promise<{ name: string; id: number }[]> {
		const currentFiles = untrack(() => fileGetter());
		const results: { name: string; id: number }[] = [];

		for (const file of currentFiles) {
			const entry = untrack(() => pending.get(fileKey(file)));
			if (!entry) continue;

			const prepared = await entry.prepare;
			const id = await entry.upload;
			if (id !== null) {
				results.push({ name: prepared.name, id });
			}
		}

		return results;
	};
}

export function createUploadEffect(
	fileGetter: () => File[]
): () => Promise<{ name: string; id: number }[]> {
	return createUploadPipeline(fileGetter);
}
