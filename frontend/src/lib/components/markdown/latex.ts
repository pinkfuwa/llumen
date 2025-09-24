import { marked } from 'marked';
import markedKatex from 'marked-katex-extension';

// TODO: add \[ \] to latex tokenizer
marked.use(
	markedKatex({
		throwOnError: false,
		nonStandard: false
	})
);

export default function initLatex() {
	console.log('inited latex');
}
