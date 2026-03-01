import { parseSync } from '../parser/block';

self.onmessage = (event: MessageEvent<{ source: string; id: number }>) => {
	const { source, id } = event.data;
	try {
		const result = parseSync(source);
		self.postMessage({ result, id });
	} catch (error) {
		let message: string;
		if (
			typeof error === 'object' &&
			error !== null &&
			'message' in error &&
			typeof (error as { message?: unknown }).message === 'string'
		) {
			message = (error as { message: string }).message;
		} else {
			message = String(error);
		}
		self.postMessage({ error: message, id });
	}
};
