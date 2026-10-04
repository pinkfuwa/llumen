import { RawAPIFetch, APIFetch } from './http.svelte';
import type { FileUploadResp, FileRefreshReq, FileRefreshResp } from './types';
import { prepareUploadFile } from './filePreparation';
import { createUploadQueue } from './uploadPipeline';
import { displayError } from '$lib/error.svelte';
import { untrack } from 'svelte';
import { token } from '$lib/rune.svelte';

const MAX_FILE_SIZE = 100 * 1024 * 1024;

export async function upload(file: File, signal?: AbortSignal): Promise<number | null> {
	const formData = new FormData();
	if (file.size > MAX_FILE_SIZE) {
		displayError('internal', 'File size exceeds the maximum limit of 100MB.');
		return null;
	}

	formData.append('size', file.size.toString());
	formData.append('file', file);

	const response = await RawAPIFetch({
		path: 'file/upload',
		body: formData,
		signal,
		token: token.value?.value
	});

	if (!response || !response.ok) {
		console.warn('Fail to upload', { file });
		return null;
	}

	const data = (await response.json()) as FileUploadResp;
	return data.id;
}

export async function refresh(fileIds: number[]): Promise<number | null> {
	if (fileIds.length === 0) return null;

	const response = await APIFetch<FileRefreshResp, FileRefreshReq>({
		path: 'file/refresh',
		body: { ids: fileIds },
		token: token.value?.value
	});

	return response?.valid_until ?? null;
}

export async function download(id: number): Promise<string | undefined> {
	const response = await RawAPIFetch<undefined>({
		path: `file/read/${encodeURIComponent(id)}`,
		method: 'GET',
		token: token.value?.value
	});

	let fail = !response || !response.ok;
	if (!fail) {
		const contentType = response!.headers.get('Content-Type')?.split(';')[0].trim().toLowerCase();
		fail = contentType === 'application/json';
	}

	if (fail) {
		console.warn('Fail to download', { id });
		return;
	}

	const blob = await response!.blob();
	return URL.createObjectURL(blob);
}

// FIXME: svelte might read async and cancel reactive context
export async function downloadCompressed(id: number): Promise<string | undefined> {
	const width = Math.max(Math.ceil(window.devicePixelRatio * screen.width), 100);
	const response = await RawAPIFetch<undefined>({
		path: `file/image/${width}/${encodeURIComponent(id)}`,
		method: 'GET',
		token: true
	});

	let fail = !response || !response.ok;
	if (!fail) {
		const contentType = response!.headers.get('Content-Type')?.split(';')[0].trim().toLowerCase();
		fail = contentType === 'application/json';
	}

	if (fail) {
		console.warn('Fail to download compressed image', { id, width });
		return;
	}

	const blob = await response!.blob();
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

/**
 * Starts uploads as selection changes within a Svelte effect scope.
 * The returned function waits for a selection snapshot in order. Removed files
 * and null upload results are omitted; other failures reject the call.
 */
export function createUploadPipeline(
	fileGetter: () => File[]
): () => Promise<{ name: string; id: number }[]> {
	const queue = createUploadQueue(async (file, signal) => {
		const prepared = await prepareUploadFile(file, signal);
		signal.throwIfAborted();
		const id = await upload(prepared, signal);
		return id === null ? null : { name: prepared.name, id };
	});

	$effect(() => {
		const files = [...fileGetter()];
		untrack(() => queue.update(files));
	});
	$effect(() => () => queue.dispose());

	return () => queue.ready(untrack(() => [...fileGetter()]));
}
