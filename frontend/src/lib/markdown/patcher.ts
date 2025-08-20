import { dev } from '$app/environment';
import type { TokensList } from 'marked';
import { Lexer } from 'marked';

/**
 * UIUpdater maintain a list of Token
 * When append, it append token to list, and only update those append
 * When replace, it replace last append token, and only update those changed
 */
export interface UIUpdater {
	reset: () => void;
	append: (tokens: TokensList) => void;
	replace: (tokens: TokensList) => void;
}

const blocktokens = [
	'space',
	'code',
	'blockquote',
	'html',
	'heading',
	'hr',
	'list',
	'listitem',
	'checkbox',
	'paragraph',
	'table',
	'tablerow',
	'tablecell'
];

const mininalFlush = 16;

export class MarkdownPatcher {
	updater: UIUpdater;
	buffer: string = '';
	content: string = '';
	lastFlush: number = 0;
	constructor(updater: UIUpdater) {
		this.updater = updater;
	}
	feed(data: string) {
		this.buffer += data;
		this.content += data;
		this.lastFlush += data.length;

		if (this.lastFlush < mininalFlush) return;
		else this.lastFlush = 0;

		const lexer = new Lexer();
		const tokens = lexer.lex(this.buffer);

		if (dev && tokens.some((x) => !blocktokens.includes(x.type)))
			throw new Error('unreachable, only blocktoken can appear at top-level');

		if (tokens.length >= 4 && !tokens[tokens.length - 2].type.startsWith('table')) {
			let first = tokens;

			let second: TokensList = [first.pop()!] as any;
			second.links = first.links;

			this.buffer = second[0].raw;

			this.updater.replace(first);
			this.updater.append(second);
		} else {
			this.updater.replace(tokens);
		}
	}
	reset() {
		this.updater.reset();
		this.buffer = '';
		this.content = '';
	}
}
