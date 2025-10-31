import { parseMarkdown, type WorkerRequest } from '../parser';

self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
	const raw = event.data;

	try {
		let token = await parseMarkdown(raw);

		self.postMessage(token);
	} catch (error) {
		self.postMessage({ error });
	}
};
