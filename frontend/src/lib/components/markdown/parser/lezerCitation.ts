import type { MarkdownConfig } from '@lezer/markdown';
import { BlockContext, Line } from '@lezer/markdown';

// Matches <citation>...</citation> blocks, extracting fields as in marked extension
const citationBlockRegex = /^<citation[^>]*>([\s\S]*?)<\/citation>/;
const fieldRegex = /<(\w+)>([\s\S]*?)<\/\1>/g;

export const lezerCitation: MarkdownConfig = {
	defineNodes: ['Citation'],
	parseBlock: [
		{
			name: 'Citation',
			parse(cx: BlockContext, line: Line) {
				const text = line.text;
				const match = citationBlockRegex.exec(text);
				if (match && match.index === 0) {
					const raw = match[0];
					const from = line.pos;
					const to = line.pos + raw.length;

					// Add the element
					cx.addElement(cx.elt('Citation', from, to));

					// Consume the line
					cx.nextLine();
					return true;
				}
				return false;
			}
		}
	]
};

export default lezerCitation;
