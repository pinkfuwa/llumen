import { marked } from 'marked';
import markedKatex from 'marked-katex-extension';

marked.use(
	markedKatex({
		throwOnError: false,
		nonStandard: true
	})
);

export default function initLatex() {
	console.log('inited latex');
}
