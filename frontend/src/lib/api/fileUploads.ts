export interface UploadedFile {
	name: string;
	id: number;
}

type PrepareAndUploadFile = (file: File, signal: AbortSignal) => Promise<UploadedFile | null>;

interface FileUpload {
	result: Promise<UploadedFile | null>;
	controller: AbortController;
}

/**
 * Keeps one result per selected File object, including null and rejected results.
 * Selecting a removed File again starts another upload.
 */
export function createFileUploads(prepareAndUploadFile: PrepareAndUploadFile) {
	const uploads = new Map<File, FileUpload>();
	let closed = false;

	function startUpload(file: File): FileUpload {
		const controller = new AbortController();
		const { signal } = controller;
		const result = new Promise<UploadedFile | null>((resolve, reject) => {
			const onAbort = () => resolve(null);
			signal.addEventListener('abort', onAbort, { once: true });
			Promise.resolve()
				.then(() => {
					signal.throwIfAborted();
					return prepareAndUploadFile(file, signal);
				})
				.then(
					(value) => resolve(signal.aborted ? null : value),
					(error) => (signal.aborted ? resolve(null) : reject(error))
				)
				.finally(() => signal.removeEventListener('abort', onAbort));
		});
		// Handle rejection before waitForUploads() is called; keep the error for that caller.
		void result.catch(() => {});
		return { controller, result };
	}

	function setFiles(files: readonly File[]) {
		if (closed) throw new Error('File uploads are closed');
		const selected = new Set(files);
		for (const [file, entry] of uploads) {
			if (!selected.has(file)) {
				uploads.delete(file);
				entry.controller.abort();
			}
		}
		for (const file of files) {
			if (!uploads.has(file)) uploads.set(file, startUpload(file));
		}
	}

	/**
	 * Copies the selected files, starts missing uploads, and waits for their results.
	 * Returns uploaded files in selection order, excluding removed files and null
	 * results. Non-cancellation errors reject with the original error.
	 */
	async function waitForUploads(files: readonly File[]): Promise<UploadedFile[]> {
		const snapshot = [...files];
		setFiles(snapshot);
		const entries = snapshot.map((file) => uploads.get(file)!);
		const results = await Promise.all(entries.map((entry) => entry.result));
		return results.filter(
			(result, index): result is UploadedFile =>
				result !== null && !entries[index].controller.signal.aborted
		);
	}

	/** Aborts uploads and prevents subsequent setFiles and waitForUploads calls. */
	function close() {
		closed = true;
		for (const entry of uploads.values()) entry.controller.abort();
		uploads.clear();
	}

	return { setFiles, waitForUploads, close };
}
