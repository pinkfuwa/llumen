import { HighlightQueue } from './queue';
import type { HighlightResponse } from './types';

let queue: HighlightQueue | undefined;

export function highlightQueue() {
	if (!queue) {
		let worker: Worker | undefined;
		queue = new HighlightQueue((request) => {
			worker ??= new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
			worker.onmessage = ({ data }: MessageEvent<HighlightResponse>) => queue!.finish(data);
			worker.onerror = () => {
				queue!.fail();
				worker?.terminate();
			};
			worker.postMessage(request);
		});
	}
	return queue;
}
