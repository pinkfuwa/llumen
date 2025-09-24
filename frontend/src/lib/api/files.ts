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
	private uploads: { promise: Promise<number | null>; file: File }[] = [];
	private abortController: AbortController = new AbortController();
	private chatId: number;
	constructor(chatId: number) {
		this.chatId = chatId;
		onDestroy(() => {
			this.abortController.abort();
		});
	}
	async getUploads(files: File[]): Promise<{ name: string; id: number }[]> {
		this.retain(files);

		let results: { name: string; id: number }[] = [];
		for (const upload of this.uploads) {
			if (
				!files.find((f) => {
					try {
						return f.name === upload.file.name && f.size === upload.file.size;
					} catch (e) {
						return false;
					}
				})
			)
				continue;

			const id = await upload.promise;
			if (id !== null) results.push({ name: upload.file.name, id });
		}

		return results;
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
