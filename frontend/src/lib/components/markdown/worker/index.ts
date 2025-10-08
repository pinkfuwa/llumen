import type { WorkerResponse, WorkerRequest } from './types';
import Semaphore from '../../../semaphore';

const worker = new Worker(new URL('./worker.ts', import.meta.url), {
	type: 'module'
});

const CACHE_SIZE = 12;

const cache = new Map<string, WorkerResponse>();
let semphore = new Semaphore();
let lexCallback: null | ((data: WorkerResponse) => void) = null;

worker.addEventListener('message', (event: MessageEvent<WorkerResponse>) => {
	const tokens = event.data;
	if (lexCallback == null) {
		console.warn(`No callback found`);
	} else lexCallback(tokens);
});

export function getCachedLex(markdown: string): WorkerResponse | null {
	if (cache.has(markdown)) return cache.get(markdown)!;
	return null;
}

export async function lex(markdown: string, shouldCache: boolean = false): Promise<WorkerResponse> {
	await semphore.acquire();

	let tokens = await new Promise<WorkerResponse>((resolve) => {
		lexCallback = resolve;
		worker.postMessage(markdown as WorkerRequest);
	});

	if (shouldCache) {
		if (cache.size >= CACHE_SIZE) {
			const firstKey = cache.keys().next().value!;
			cache.delete(firstKey);
		}
		cache.set(markdown, tokens);
	}

	semphore.release();

	return tokens;
}
