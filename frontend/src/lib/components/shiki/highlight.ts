import type { ShikiWorkerRequest, ShikiWorkerResponse } from './types';
import Semaphore from '$lib/semaphore';

const worker = new Worker(new URL('./worker.ts', import.meta.url), {
	type: 'module'
});

let semphore = new Semaphore();
let renderCallback: null | ((data: ShikiWorkerResponse) => void) = null;

worker.addEventListener('message', (event: MessageEvent<ShikiWorkerResponse>) => {
	const data = event.data;
	if (renderCallback == null) {
		console.warn(`No callback found`);
	} else {
		renderCallback(data);
	}
});

export async function highlight(
	code: string,
	lang: string,
	theme: 'light' | 'dark'
): Promise<string> {
	await semphore.acquire();

	let { html, error } = await new Promise<ShikiWorkerResponse>((resolve) => {
		renderCallback = resolve;
		const request: ShikiWorkerRequest = { code, lang, theme };
		worker.postMessage(request);
	});

	semphore.release();

	if (error) {
		console.warn(error);
		throw error;
	}

	return html!;
}
