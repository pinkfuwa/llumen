import { onDestroy } from 'svelte';
import { RawAPIFetch } from './state/errorHandle';
import type { FileUploadResp } from './types';

export async function upload(file: File, chatId: number, signal?: AbortSignal) {
	const formData = new FormData();
	formData.append('chat_id', chatId.toString());
	formData.append('file', file);

	const response = await RawAPIFetch('file/upload', formData, 'POST', signal);

	if (!response.ok) {
		console.warn('Fail to upload', { file });
		return null;
	}

	const data = (await response.json()) as FileUploadResp;
	return data.id;
}

export async function download(id: number) {
	const response = await RawAPIFetch('file/download/' + encodeURIComponent(id), null);

	if (!response.ok) {
		console.warn('Fail to download', { id });
		return;
	}

	const blob = await response.blob();
	return URL.createObjectURL(blob);
}

export class UploadManager {
	uploads: { promise: Promise<number | null>; file: File }[] = [];
	abortController: AbortController = new AbortController();
	chatId: number;
	constructor(chatId: number) {
		this.chatId = chatId;
		onDestroy(() => {
			this.abortController.abort();
		});
	}
	retain(files: File[]) {
		const newUploads = files
			.filter((file) => {
				return !this.uploads.find((u) => u.file.name === file.name && u.file.size === file.size);
			})
			.map((file) => ({
				promise: upload(file, this.chatId, this.abortController.signal),
				file
			}));
		this.uploads = [...this.uploads, ...newUploads];
	}
}
