import { dev } from '$app/environment';
import DOMPurify from 'isomorphic-dompurify';
import Worker from './worker?worker';

let worker: Worker | undefined;
if (!dev) worker = new Worker();

export async function renderPlain(html: string): Promise<string> {
	if (!worker) return (await import('./plain')).renderPlainMarkdown(html);

	worker.postMessage({ type: 'render', data: html });
	const result = await new Promise<string>((resolve) => {
		worker.onmessage = (event) => {
			resolve(event.data.data);
		};
	});
	return result;
}

function applyStyle(htmlString: string): string {
	const parser = new DOMParser();
	const doc = parser.parseFromString(htmlString, 'text/html');

	const styleMap = new Map<string, string>([
		['h1', 'text-2xl font-bold'],
		['h2', 'text-xl font-bold'],
		['h3', 'text-lg font-bold'],
		['pre', 'rounded-md border border-outline p-2 border-radius-md overflow-x-auto'],
		['p', 'items-center'],
		['ul', 'list-disc ml-6']
	]);

	styleMap.forEach((classValue, tagName) => {
		const elements = doc.querySelectorAll(tagName);
		elements.forEach((element) => {
			const currentClass = element.getAttribute('class') || '';
			element.setAttribute('class', `${currentClass} ${classValue}`.trim());
		});
	});

	return doc.documentElement.outerHTML;
}

export async function render(html: string): Promise<string> {
	const unsafeHTML = await renderPlain(html);
	const unstyledHTML = DOMPurify.sanitize(unsafeHTML);
	return applyStyle(unstyledHTML);
}
