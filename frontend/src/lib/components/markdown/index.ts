import { marked } from 'marked';
import initCitation from './citation';
import initLatex from './latex';
import Markdown from './Root.svelte';
import type { TokensList } from 'marked';

export function init() {
	initLatex();
	initCitation();
}
export { Markdown };

const MAX_CACHE_SIZE = 10;
const lexerCache = new Map<string, any>();

export function lex(source: string): Promise<TokensList> {
	if (lexerCache.has(source)) {
		return Promise.resolve(lexerCache.get(source));
	}

	return new Promise((resolve) =>
		setTimeout(() => {
			const tokens = marked.lexer(source);

			if (lexerCache.size > MAX_CACHE_SIZE) {
				const firstKey = lexerCache.keys().next().value!;
				lexerCache.delete(firstKey);
			}
			lexerCache.set(source, tokens);

			resolve(tokens);
		}, 0)
	);
}

export function heatCache(source: string) {
	if (!lexerCache.has(source)) {
		const tokens = marked.lexer(source);
		if (lexerCache.size > MAX_CACHE_SIZE) {
			const firstKey = lexerCache.keys().next().value!;
			lexerCache.delete(firstKey);
		}
		lexerCache.set(source, tokens);
	}
}
