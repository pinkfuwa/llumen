import type { WorkerToken } from './types';

let importInstance: null | Promise<typeof import('./parser')> = null;

export async function parseMarkdown(source: string): Promise<WorkerToken[]> {
	if (importInstance === null) {
		importInstance = import('./parser');
	}
	const parser = await importInstance;
	return parser.parseMarkdown(source);
}

export type { WorkerToken };
export type { WorkerResponse, WorkerRequest } from './types';
