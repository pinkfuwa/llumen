export interface UploadedFile {
	name: string;
	id: number;
}

type ProcessFile = (file: File, signal: AbortSignal) => Promise<UploadedFile | null>;

interface PendingUpload {
	result: Promise<UploadedFile | null>;
	controller: AbortController;
}

export function createUploadQueue(processFile: ProcessFile) {
	const pending = new Map<File, PendingUpload>();
	let disposed = false;

	function start(file: File): PendingUpload {
		const controller = new AbortController();
		const { signal } = controller;
		const result = new Promise<UploadedFile | null>((resolve, reject) => {
			const onAbort = () => resolve(null);
			signal.addEventListener('abort', onAbort, { once: true });
			Promise.resolve()
				.then(() => {
					signal.throwIfAborted();
					return processFile(file, signal);
				})
				.then(
					(value) => resolve(signal.aborted ? null : value),
					(error) => (signal.aborted ? resolve(null) : reject(error))
				)
				.finally(() => signal.removeEventListener('abort', onAbort));
		});
		// Uploads start before ready() observes their promises.
		void result.catch(() => {});
		return { controller, result };
	}

	function update(files: readonly File[]) {
		if (disposed) throw new Error('Upload pipeline is disposed');
		const selected = new Set(files);
		for (const [file, entry] of pending) {
			if (!selected.has(file)) {
				pending.delete(file);
				entry.controller.abort();
			}
		}
		for (const file of files) {
			if (!pending.has(file)) pending.set(file, start(file));
		}
	}

	/**
	 * Reconciles a selection snapshot and returns successes in selection order.
	 * Removed files and null results are omitted; other failures reject.
	 */
	async function ready(files: readonly File[]): Promise<UploadedFile[]> {
		const snapshot = [...files];
		update(snapshot);
		const entries = snapshot.map((file) => pending.get(file)!);
		const results = await Promise.all(entries.map((entry) => entry.result));
		return results.filter(
			(result, index): result is UploadedFile =>
				result !== null && !entries[index].controller.signal.aborted
		);
	}

	function dispose() {
		disposed = true;
		for (const entry of pending.values()) entry.controller.abort();
		pending.clear();
	}

	return { update, ready, dispose };
}
