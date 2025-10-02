import type { WorkerPayload, WorkerRequest, WorkerResponse } from './types';

const worker = new Worker(new URL('./worker.ts', import.meta.url), {
	type: 'module'
});

const CACHE_SIZE = 6;

const cache = new Map<string, WorkerPayload>();
const pending = new Map<string, Promise<WorkerPayload>>();

const lexCallbacks: Map<string, (data: WorkerPayload) => void> = new Map();

worker.addEventListener('message', (event: MessageEvent<WorkerResponse>) => {
	const { input: markdown, data: tokens } = event.data;
	if (cache.size >= CACHE_SIZE) {
		const firstKey = cache.keys().next().value!;
		cache.delete(firstKey);
	}
	cache.set(markdown, tokens);
	pending.delete(markdown);

	if (!lexCallbacks.has(markdown)) {
		console.warn(`No callback found for markdown: ${markdown.slice(undefined, 20)}`);
	}
	lexCallbacks.get(markdown)!(tokens);
	lexCallbacks.delete(markdown);
});

export const lexer = (markdown: string): Promise<WorkerResponse['data']> => {
	if (cache.has(markdown)) return Promise.resolve(cache.get(markdown)!);

	const pendingPromise = pending.get(markdown);
	if (pendingPromise) return pendingPromise;

	const promise = new Promise<WorkerResponse['data']>((resolve) => {
		worker.postMessage(markdown as WorkerRequest);
		lexCallbacks.set(markdown, resolve);
	});

	pending.set(markdown, promise);

	return promise;
};
