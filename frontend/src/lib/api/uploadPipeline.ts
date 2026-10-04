export interface UploadedFile {
	name: string;
	id: number;
}

type ProcessFile = (file: File, signal: AbortSignal) => Promise<UploadedFile | null>;

interface PendingUpload {
	result: Promise<UploadedFile | null>;
	controller: AbortController;
}

function fileKey(file: File): string {
	return `${file.name}-${file.size}`;
}

export function createUploadQueue(processFile: ProcessFile) {
	const pending = new Map<string, PendingUpload>();

	function update(files: readonly File[]) {
		const keys = new Set(files.map(fileKey));
		for (const [key, entry] of pending) {
			if (!keys.has(key)) {
				pending.delete(key);
				entry.controller.abort();
			}
		}
		for (const file of files) {
			const key = fileKey(file);
			if (pending.has(key)) continue;
			const controller = new AbortController();
			pending.set(key, { controller, result: processFile(file, controller.signal) });
		}
	}

	async function ready(files: readonly File[]): Promise<UploadedFile[]> {
		const results: UploadedFile[] = [];
		for (const file of files) {
			const entry = pending.get(fileKey(file));
			if (!entry) continue;
			const result = await entry.result;
			if (result) results.push(result);
		}
		return results;
	}

	function dispose() {
		for (const entry of pending.values()) entry.controller.abort();
	}

	return { update, ready, dispose };
}
