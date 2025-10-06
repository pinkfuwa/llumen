import type { MarkedExtension } from 'marked';

const bracketInlineRegex = /^(\\\[)(?!\$)((?:\\.|[^\\\n])*?(?:\\.|[^\\\n\$]))\\\]/;
const bracketBlockRegex = /^(\\\[)\n((?:\\[^]|[^\\])+?)\n\\\](?:\n|$)/;
const parenthesesInlineRegex = /^(\\\()(?!\$)((?:\\.|[^\\\n])*?(?:\\.|[^\\\n\$]))\\\)/;
const parenthesesBlockRegex = /^(\\\()\n((?:\\[^]|[^\\])+?)\n\\\)(?:\n|$)/;
const dollarInlineRule =
	/^(\${1,2})(?!\$)((?:\\.|[^\\\n])*?(?:\\.|[^\\\n\$]))\1(?=[\s?!\.,:？！。，：]|$)/;
const dollarBlockRule = /^(\${1,2})\n((?:\\[^]|[^\\])+?)\n\1(?:\n|$)/;

const Latex: MarkedExtension = {
	extensions: [
		{
			name: 'inlineKatex',
			level: 'inline',
			start(src) {
				let index;
				let indexSrc = src;

				while (indexSrc) {
					index = indexSrc.indexOf('$') || indexSrc.indexOf('\\[') || indexSrc.indexOf('\\(');
					if (index === -1) return;

					const f = index > -1;
					if (f) {
						const possibleKatex = indexSrc.substring(index);

						if (
							possibleKatex.match(bracketInlineRegex) ||
							possibleKatex.match(parenthesesInlineRegex) ||
							possibleKatex.match(dollarInlineRule)
						) {
							return index;
						}
					}

					indexSrc = indexSrc
						.substring(index + 1)
						.replace(/^\\\[/, '')
						.replace(/^\$+/, '')
						.replace(/^\\\(/, '');
				}
			},
			tokenizer(src, tokens) {
				const match =
					src.match(bracketInlineRegex) ||
					src.match(parenthesesInlineRegex) ||
					src.match(dollarInlineRule);
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
				const match =
					bracketBlockRegex.exec(src) ||
					parenthesesBlockRegex.exec(src) ||
					dollarBlockRule.exec(src);
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
};

export default Latex;
