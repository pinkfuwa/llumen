import { parse, walkTree } from '../lexer';

self.onmessage = async (event) => {
	const { source, id } = event.data;
	try {
		const tree = await parse(source);

		const ast = await walkTree(tree, source);
		self.postMessage({ ast, id });
	} catch (error) {
		console.error(error);
		// let message: string;
		// if (
		// 	typeof error === 'object' &&
		// 	error !== null &&
		// 	'message' in error &&
		// 	typeof (error as { message?: unknown }).message === 'string'
		// ) {
		// 	message = (error as { message: string }).message;
		// } else {
		// 	message = String(error);
		// }
		// self.postMessage({ error: message, id });
	}
};
