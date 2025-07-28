import { Marked } from 'marked';
import styleExt from './style';
import emojiExt from './emoji';
import purifyExt from './purify';
import getShikiExt from './shiki';
import { useTheme } from '$lib/store';
import { get } from 'svelte/store';

function themedMark(theme: 'light' | string): Marked {
	let marked = new Marked();

	marked.use(emojiExt);
	marked.use(getShikiExt(theme == 'light' ? 'github-light' : 'github-dark'));
	marked.use(styleExt);
	marked.use(purifyExt);

	return marked;
}

/**
 * Direct render of markdown content
 * @param content
 * @returns
 */
export async function render(content: string): Promise<string> {
	let theme = useTheme();
	let marked = themedMark(get(theme));
	const html = marked.parse(content);

	return html instanceof Promise ? await html : html;
}

render(['# test code block', '```javascript', "console.log('hello world');", '```'].join('\n'));
