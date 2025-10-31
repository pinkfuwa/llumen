import type { WorkerResponse } from '../parser/types';
import Semaphore from '../../../semaphore';

const worker = new Worker(new URL('./worker.ts', import.meta.url), {
	type: 'module'
});

let semaphore = new Semaphore();
let lexCallback: null | ((data: WorkerResponse) => void) = null;

worker.addEventListener('message', (event: MessageEvent<WorkerResponse>) => {
	const tokens = event.data;
	if (lexCallback == null) {
		console.warn(`No callback found`);
	} else lexCallback(tokens);
});

export async function parseMarkdown(markdown: string): Promise<WorkerResponse> {
	await semaphore.acquire();

	let tokens = await new Promise<WorkerResponse>((resolve) => {
		lexCallback = resolve;
		worker.postMessage({ markdown });
	});

	semaphore.release();

	return tokens;
}
