import { tokenizeCode } from './tokenize';
import type { HighlightRequest, HighlightResponse } from './types';

self.onmessage = ({ data }: MessageEvent<HighlightRequest>) => {
	let tokens: HighlightResponse['tokens'] = [];
	try {
		tokens = tokenizeCode(data.text, data.language);
	} catch {
		// A grammar failure must not prevent the independent source renderer from working.
	}
	self.postMessage({ id: data.id, tokens } satisfies HighlightResponse);
};
