import Markdown from './Root.svelte';
import { lex } from './worker';
export { Markdown };

export function heatMarkdownCache(source: string) {
	lex(source, true);
}
