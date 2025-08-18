import { marked } from 'marked';

interface Entry<D> {
	source: string;
	key?: string;
	tokens: D;
}

class FIFOCache<D, O = void> {
	maxSize: number;
	entries: Entry<D>[] = [];
	f: (source: string, option?: O) => D;

	constructor(maxSize: number, f: (source: string, option?: O) => D) {
		this.maxSize = maxSize;
		this.f = f;
	}

	run(source: string, option?: O) {
		const key = option ? JSON.stringify(option) : undefined;
		const item = this.entries.find((entry) => entry.source == source && entry.key == key);
		if (item) return item.tokens;

		const tokens = this.f(source, option);
		if (this.entries.length > 20) this.entries.pop();

		this.entries.unshift({
			source,
			key,
			tokens
		});
		return tokens;
	}
}

const markLexer = new marked.Lexer();
export const lexer = new FIFOCache(10, (x) => markLexer.lex(x));
