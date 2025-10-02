import initCitation from './citation';
import initLatex from './latex';
import Markdown from './Root.svelte';
import { lexer as lex } from './worker';

export function init() {
	initLatex();
	initCitation();
}
export { Markdown, lex };
