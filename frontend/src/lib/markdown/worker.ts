import { render } from './render';
console.log(`markdown web worker started`);

self.onmessage = (event) => {
	const task = event.data as string;
	const data = render(task);
	self.postMessage({ data });
};
