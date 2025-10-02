import Markdown from './Root.svelte';
import { lex } from './worker';
export { Markdown };

export async function heatMarkdownCache(source: string) {
	await lex(source, true);
}
