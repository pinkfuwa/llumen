import { marked } from 'marked';
import type { WorkerRequest, WorkerResponse } from './types';
import Latex from './latex';
import Citation from './citation';

marked.use(Latex);
marked.use(Citation);

self.onmessage = (event: MessageEvent<WorkerRequest>) => {
	const markdown = event.data;

	const tokens = marked.lexer(markdown);
	self.postMessage({ input: markdown, data: tokens } as WorkerResponse);
};
