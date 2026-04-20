import type { AstNode, ParseResult } from '../parser/types';

let worker: Worker | null = null;
let requestId = 0;

const pending = new Map<
	number,
	{ resolve: (result: AstNode[]) => void; reject: (err: Error) => void }
>();

function ensureWorker(): Worker {
	if (!worker) {
		worker = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
		worker.onmessage = (event: MessageEvent<{ nodes?: AstNode[]; error?: string; id: number }>) => {
			const { nodes, error, id } = event.data;
			const cb = pending.get(id);
			if (cb) {
				pending.delete(id);
				if (error) {
					cb.reject(new Error(error));
				} else if (nodes) {
					cb.resolve(nodes);
				}
			}
		};
	}
	return worker;
}

export function parseMarkdown(source: string): Promise<AstNode[]> {
	return new Promise((resolve, reject) => {
		const id = ++requestId;
		pending.set(id, { resolve, reject });
		ensureWorker().postMessage({ source, id });
	});
}

export async function parseMarkdownAsync(source: string): Promise<ParseResult> {
	const { parseAsync } = await import('../parser/index');
	return parseAsync(source);
}
