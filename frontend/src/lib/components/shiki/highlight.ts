import type { ShikiWorkerRequest } from './types';
import Semaphore from '$lib/semaphore';

const worker = new Worker(new URL('./worker.ts', import.meta.url), {
	type: 'module'
});

let semphore = new Semaphore();
let renderCallback: null | ((data: string) => void) = null;

worker.addEventListener('message', (event: MessageEvent<string>) => {
	const html = event.data;
	if (renderCallback == null) {
		console.warn(`No callback found`);
	} else renderCallback(html);
});

export async function highlight(
	code: string,
	lang: string,
	theme: 'light' | 'dark'
): Promise<string> {
	await semphore.acquire();

	let html = await new Promise<string>((resolve) => {
		renderCallback = resolve;
		const request: ShikiWorkerRequest = { code, lang, theme };
		worker.postMessage(request);
	});

	semphore.release();

	return html;
}
