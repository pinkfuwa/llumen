import { dev } from '$app/environment';
import type { TokensList } from 'marked';
import { lex } from './worker';

/**
 * UIUpdater maintain a list of Token
 * When append, it append token to list, and only update those append
 * When replace, it replace last append token, and only update those changed
 */
export interface UIUpdater {
	/**
	 * When empty token are received, tick is called
	 */
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
	'blockKatex',
	'citation'
];

const flushThreshold = 9;

export class MarkdownPatcher {
	updater: UIUpdater;
	// content that's not rendered yet
	buffer: string = '';
	// content of last chunk
	lastChunk: string = '';
	// total content
	content: string = '';
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
	async feed(data: string) {
		this.content += data;
		this.buffer += data;

		if (this.flushWeight < flushThreshold) return;

		this.lastChunk += this.buffer;
		this.buffer = '';

		const tokens = await lex(this.lastChunk);

		if (dev && tokens.some((x) => !blocktokens.includes(x.type))) {
			console.warn('only blocktoken can appear at top-level');
		}

		const overlapTokens = ['table', 'citation'];
		if (tokens.length >= 3 && !tokens.slice(1).some((x) => overlapTokens.includes(x.type))) {
			let first = tokens;

			let second: TokensList = [first.pop()!] as any;
			second.links = first.links;

			this.lastChunk = second[0].raw;

			this.updater.replace(first);
			this.updater.append(second);
		} else {
			this.updater.replace(tokens);
		}
	}
	reset() {
		this.updater.reset();
		this.lastChunk = '';
		this.content = '';
		this.buffer = '';
	}
}
