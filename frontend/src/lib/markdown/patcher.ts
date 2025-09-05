import { dev } from '$app/environment';
import type { TokensList } from 'marked';
import { marked } from 'marked';

/**
 * UIUpdater maintain a list of Token
 * When append, it append token to list, and only update those append
 * When replace, it replace last append token, and only update those changed
 */
export interface UIUpdater {
	/**
	 * When empty token are received, tick is called
	 */
	tick: () => void;
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
	'tablecell',
	'blockKatex'
];

const flushThreshold = 9;

export class MarkdownPatcher {
	updater: UIUpdater;
	buffer: string = '';
	content: string = '';
	lastFlush: number = 0;
	constructor(updater: UIUpdater) {
		this.updater = updater;
	}
	get flushWeight(): number {
		let weight = 0;
		for (let i = 0; i < this.buffer.length; i++) {
			const code = this.buffer.charCodeAt(i);
			if (code >= 0x4e00 && code <= 0x9fa5)
				weight += 4; // Chinese char
			else if (code < 128)
				weight += 1; // English char
			else weight += 2; // Other unicode char
		}
		return weight;
	}
	feed(data: string) {
		this.buffer += data;
		this.content += data;
		this.lastFlush += data.length;

		if (this.flushWeight < flushThreshold) return;
		else this.lastFlush = 0;

		const tokens = marked.lexer(this.buffer);

		if (dev && tokens.some((x) => !blocktokens.includes(x.type)))
			throw new Error('unreachable, only blocktoken can appear at top-level');

		if (tokens.length >= 3 && !tokens[tokens.length - 2].type.startsWith('table')) {
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
