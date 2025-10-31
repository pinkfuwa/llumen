import type { MarkdownConfig } from '@lezer/markdown';
import { BlockContext, InlineContext, Line } from '@lezer/markdown';

// Regexes for block and inline latex/math
const bracketInlineRegex = /^(\\\[)(?!\$)((?:\\.|[^\\\n])*?(?:\\.|[^\\\n\$]))\\\]/;
const bracketBlockRegex = /^(\\\[)\n((?:\\[^]|[^\\])+?)\n\\\](?:\n|$)/;
const parenthesesInlineRegex = /^(\\\()(?!\$)((?:\\.|[^\\\n])*?(?:\\.|[^\\\n\$]))\\\)/;
const parenthesesBlockRegex = /^(\\\()\n((?:\\[^]|[^\\])+?)\n\\\)(?:\n|$)/;
const dollarInlineRule =
	/^(\${1,2})(?!\$)((?:\\.|[^\\\n])*?(?:\\.|[^\\\n\$]))\1(?=[\s?!\.,:？！。，：]|$)/;
const dollarBlockRule = /^(\${1,2})\n((?:\\[^]|[^\\])+?)\n\1(?:\n|$)/;

export const lezerLatex: MarkdownConfig = {
	defineNodes: ['BlockKatex', 'InlineKatex'],
	parseBlock: [
		{
			name: 'BlockKatex',
			parse(cx: BlockContext, line: Line) {
				const src = line.text;

				// Try all block patterns
				const match =
					bracketBlockRegex.exec(src) ||
					parenthesesBlockRegex.exec(src) ||
					dollarBlockRule.exec(src);

				if (match) {
					const raw = match[0];
					const from = line.pos;
					const to = line.pos + raw.length;

					// Add the element
					cx.addElement(cx.elt('BlockKatex', from, to));

					// Consume the line
					cx.nextLine();
					return true;
				}
				return false;
			}
		}
	],
	parseInline: [
		{
			name: 'InlineKatex',
			parse(cx: InlineContext, next: number, pos: number) {
				const src = cx.text.slice(pos);

				// Try all inline patterns
				const match =
					bracketInlineRegex.exec(src) ||
					parenthesesInlineRegex.exec(src) ||
					dollarInlineRule.exec(src);

				if (match) {
					const raw = match[0];
					const to = pos + raw.length;

					// Add the element
					cx.addElement(cx.elt('InlineKatex', pos, to));
					return to;
				}
				return -1;
			}
		}
	]
};

export default lezerLatex;
