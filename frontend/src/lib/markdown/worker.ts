import { renderPlainMarkdown } from './plain';
console.log(`markdown web worker started`);

self.onmessage = async (event) => {
	const data = await renderPlainMarkdown(event.data.data as string);
	self.postMessage({ data });
};
