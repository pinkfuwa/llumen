import type { CodeToken } from './types';

type PaintedRange = { range: Range; highlight: Highlight; node: Text; start: number; end: number };

export type CodePaint = { clear(): void; refresh(): void };

export function paintTokens(element: HTMLElement, tokens: CodeToken[]): CodePaint {
	if (!globalThis.CSS?.highlights || typeof Highlight === 'undefined') {
		return { clear() {}, refresh() {} };
	}
	const lines = Array.from(element.querySelectorAll<HTMLElement>('[data-code-offset]'));
	const painted: PaintedRange[] = [];
	let lineIndex = 0;
	for (const token of tokens) {
		while (lineIndex < lines.length) {
			const line = lines[lineIndex];
			const offset = Number(line.dataset.codeOffset);
			if (offset + (line.textContent?.length ?? 0) > token.start) break;
			lineIndex++;
		}
		for (let index = lineIndex; index < lines.length; index++) {
			const line = lines[index];
			const offset = Number(line.dataset.codeOffset);
			if (offset >= token.end) break;
			const node = line.firstChild;
			if (!(node instanceof Text)) continue;
			const start = Math.max(0, token.start - offset);
			const end = Math.min(node.length, token.end - offset);
			if (end <= start) continue;
			const range = document.createRange();
			range.setStart(node, start);
			range.setEnd(node, end);
			const name = `syntax-${token.type}`;
			let highlight = CSS.highlights.get(name);
			if (!highlight) {
				highlight = new Highlight();
				CSS.highlights.set(name, highlight);
			}
			highlight.add(range);
			painted.push({ range, highlight, node, start, end });
		}
	}
	return {
		clear() {
			for (const { range, highlight } of painted) highlight.delete(range);
			for (const token of tokens) {
				const name = `syntax-${token.type}`;
				if (CSS.highlights.get(name)?.size === 0) CSS.highlights.delete(name);
			}
		},
		refresh() {
			// Replacing character data collapses live ranges even when the text node survives.
			for (const { range, node, start, end } of painted) {
				if (
					range.startContainer !== node ||
					range.startOffset !== start ||
					range.endContainer !== node ||
					range.endOffset !== end
				) {
					range.setStart(node, start);
					range.setEnd(node, end);
				}
			}
		}
	};
}
