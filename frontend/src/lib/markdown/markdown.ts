// import Worker from './worker?worker';

// const worker = new Worker();

// export async function render(html: string): Promise<string> {
// 	worker.postMessage({ type: 'render', data: html });
// 	const result = await new Promise<string>((resolve) => {
// 		worker.onmessage = (event) => {
// 			resolve(event.data);
// 		};
// 	});
// 	return result;
// }
export { render } from './render';
