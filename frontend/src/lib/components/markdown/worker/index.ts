let worker = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
let requestId = 1;

const pending = new Map<number, (ast: any) => void>();

worker.onmessage = (event) => {
	const { ast, id } = event.data;
	const cb = pending.get(id);
	if (cb) {
		cb(ast);
		pending.delete(id);
	}
};

export async function parseAst(source: string): Promise<any> {
	return new Promise((resolve) => {
		const id = ++requestId;
		pending.set(id, resolve);
		worker!.postMessage({ source, id });
	});
}
