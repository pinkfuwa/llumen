import { parseAsync, type ParseResult } from '../lexer';

let worker: Worker | null = null;
let requestId = 1;

const pending = new Map<number, (result: ParseResult) => void>();

function ensureWorker(): Worker {
	if (!worker) {
		worker = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
		worker.onmessage = (event) => {
			const { result, id } = event.data;
			const cb = pending.get(id);
			if (cb) {
				cb(result);
				pending.delete(id);
			}
		};
	}
	return worker;
}

/**
 * Parse markdown using web worker (for non-incremental parsing)
 */
export async function parseMarkdown(source: string): Promise<ParseResult> {
	return new Promise((resolve) => {
		const id = ++requestId;
		pending.set(id, resolve);
		ensureWorker().postMessage({ source, id });
	});
}

/**
 * Parse markdown in main thread using async import (for code splitting)
 */
export async function parseMarkdownAsync(source: string): Promise<ParseResult> {
	return parseAsync(source);
}
