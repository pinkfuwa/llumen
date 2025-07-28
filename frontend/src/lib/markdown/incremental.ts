import type { Marked } from 'marked';
import { themedMark } from './render';
import type { Token } from 'marked';

export class incrementalMark {
	marked: Marked;
	remainingToken: string = '';
	key = 1;
	constructor(theme: string) {
		this.marked = themedMark(theme);
	}
	/**
	 *
	 * @param chunk
	 * @returns [old chunk, several new chunk]
	 */
	nextChunk(chunk: string): Array<{ key: number; html: string }> {
		this.remainingToken += chunk;
		let ast = this.marked.lexer(this.remainingToken) as Token[];
		for (let i = 0; i < ast.length - 1; i++) {
			this.remainingToken = this.remainingToken.replace(ast[i].raw, '');
		}
		let parser = this.marked.Parser;

		return ast.map((x) => ({ key: this.key++, html: parser.parse([x]) }));
	}
	flush(): string {
		if (this.remainingToken == '') return '';
		return this.marked.parse(this.remainingToken) as string;
	}
}
