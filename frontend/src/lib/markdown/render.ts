import { Marked } from 'marked';
import styleExt from './style';
import emojiExt from './emoji';
import purifyExt from './purify';
import getShikiExt from './shiki';
import { useTheme } from '$lib/store';
import { get } from 'svelte/store';

export function themedMark(theme: 'light' | string): Marked {
	let marked = new Marked({ async: false });

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
export function render(content: string): string {
	let theme = useTheme();
	let marked = themedMark(get(theme));

	return marked.parse(content) as string;
}
