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
	append: (tokens: TokensList & { monochrome?: boolean }) => void;
	replace: (tokens: TokensList & { monochrome?: boolean }) => void;
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

export class MarkdownPatcher {
	updater: UIUpdater;
	buffer: string = '';
	content: string = '';
	lexer = new Lexer();
	constructor(updater: UIUpdater) {
		this.updater = updater;
	}
	feed(data: string) {
		this.buffer += data;
		this.content += data;
		const tokens = this.lexer.lex(this.buffer);

		if (dev && tokens.some((x) => !blocktokens.includes(x.type)))
			throw new Error('unreachable, only blocktoken can appear at top-level');

		if (tokens.length > 1) {
			let first = tokens;

			let second: TokensList = [first.pop()!] as any;
			second.links = first.links;

			this.buffer = second[0].raw;

			this.updater.replace({ ...first, monochrome: true });
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
