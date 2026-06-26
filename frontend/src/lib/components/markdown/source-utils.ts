const LATEX_PAIRS: [string, string][] = [
	['$$', '$$'],
	['$', '$'],
	['\\(', '\\)'],
	['\\[', '\\]']
];

export function expandSourceRange(src: string, start: number, end: number): [number, number] {
	for (const [open, close] of LATEX_PAIRS) {
		for (const tryStart of [start, start + 1]) {
			if (tryStart < open.length || tryStart > src.length) continue;
			if (src.slice(tryStart - open.length, tryStart) !== open) continue;

			for (const tryEnd of [end, end - 1]) {
				if (tryEnd < 0 || tryEnd + close.length > src.length) continue;
				if (src.slice(tryEnd, tryEnd + close.length) !== close) continue;
				return [tryStart - open.length, tryEnd + close.length];
			}
		}
	}

	const delim = new Set(['`', '*', '_', '~']);
	let realEnd = end;
	while (realEnd > 0 && delim.has(src[realEnd - 1])) realEnd--;
	let left = 0;
	while (start - left - 1 >= 0 && delim.has(src[start - left - 1])) left++;
	let right = 0;
	while (realEnd + right < src.length && delim.has(src[realEnd + right])) right++;
	const n = Math.min(left, right);
	if (n === 0) return [start, end];
	return [start - n, realEnd + n];
}

export function getSourceOffset(
	node: Node,
	offset: number,
	containerRef: HTMLElement
): number | null {
	let el: Element | null =
		node.nodeType === Node.TEXT_NODE ? node.parentElement : (node as Element);
	while (el && el !== containerRef) {
		const startAttr = el.getAttribute('data-offset-start');
		const endAttr = el.getAttribute('data-offset-end');
		if (startAttr === null || endAttr === null) {
			el = el.parentElement;
			continue;
		}
		const s = Number(startAttr);
		const e = Number(endAttr);
		if (isNaN(s) || isNaN(e) || s >= e) {
			el = el.parentElement;
			continue;
		}
		if (node.nodeType === Node.TEXT_NODE) {
			return s + offset;
		}
		if (offset === 0) {
			return s;
		}
		if (offset >= el.childNodes.length) {
			return e;
		}
		const child = el.childNodes[offset];
		if (child) {
			return getSourceOffset(child, 0, containerRef);
		}
		return s;
	}
	return null;
}
