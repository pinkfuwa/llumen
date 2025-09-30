import { marked } from 'marked';
import markedKatex from 'marked-katex-extension';

const bracketLatexInlineRegex = /^(\\\[)(?!\$)((?:\\.|[^\\\n])*?(?:\\.|[^\\\n\$]))\\\]/;
const bracketLatexBlockRegex = /^(\\\[)\n((?:\\[^]|[^\\])+?)\n\\\](?:\n|$)/;
const parenthesesLatexInlineRegex = /^(\\\()(?!\$)((?:\\.|[^\\\n])*?(?:\\.|[^\\\n\$]))\\\)/;
const parenthesesLatexBlockRegex = /^(\\\()\n((?:\\[^]|[^\\])+?)\n\\\)(?:\n|$)/;

export function latexTrim(i: string) {
	return i
		.replace(/^\$\$\s*/gm, '')
		.replace(/\s*\$\$$/gm, '')
		.replace(/^\\\[\s*/gm, '')
		.replace(/\s*\\\]$/gm, '')
		.replace(/^\\\(\s*/gm, '')
		.replace(/\s*\\\)$/gm, '');
}

marked.use({
	extensions: [
		{
			name: 'inlineKatex',
			level: 'inline',
			start(src) {
				let index;
				let indexSrc = src;

				while (indexSrc) {
					index = indexSrc.indexOf('$');
					if (index === -1) return;

					const f = index > -1;
					if (f) {
						const possibleKatex = indexSrc.substring(index);

						if (
							possibleKatex.match(bracketLatexInlineRegex) ||
							possibleKatex.match(parenthesesLatexInlineRegex)
						) {
							return index;
						}
					}

					indexSrc = indexSrc.substring(index + 1).replace(/^\\\[/, '');
				}
			},
			tokenizer(src, tokens) {
				const match = src.match(bracketLatexInlineRegex) || src.match(parenthesesLatexInlineRegex);
				if (match) {
					return {
						type: 'inlineKatex',
						raw: match[0],
						text: match[2].trim(),
						displayMode: match[1].length === 2
					};
				}
			}
		},
		{
			name: 'blockKatex',
			level: 'block',
			tokenizer(src: string) {
				const match = bracketLatexBlockRegex.exec(src) || parenthesesLatexBlockRegex.exec(src);
				if (match) {
					return {
						type: 'blockKatex',
						raw: match[0],
						text: match[2].trim(),
						displayMode: match[1].length === 2
					};
				}
			}
		}
	]
});

marked.use(
	markedKatex({
		throwOnError: false,
		nonStandard: false
	})
);

export default function initLatex() {
	console.log('inited latex');
}
