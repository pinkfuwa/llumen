import { afterEach, describe, expect, it, vi } from 'vitest';
import { paintTokens } from './paint';

afterEach(() => {
	vi.unstubAllGlobals();
	document.body.replaceChildren();
});

describe('paint-only syntax decoration', () => {
	it("preserves source DOM and removes only the destroyed block's ranges", () => {
		const highlights = new Map<string, Set<Range>>();
		vi.stubGlobal('CSS', { highlights });
		vi.stubGlobal('Highlight', Set);
		const first = document.createElement('pre');
		first.innerHTML =
			'<span data-code-offset="0">😀 x</span><span data-code-offset="5">\t你好</span>';
		const second = first.cloneNode(true) as HTMLElement;
		document.body.append(first, second);
		const original = first.innerHTML;
		const node = first.firstChild;
		const tokens = [{ start: 0, end: 8, type: 'str' as const }];
		const clearFirst = paintTokens(first, tokens);
		const clearSecond = paintTokens(second, tokens);
		expect(first.innerHTML).toBe(original);
		expect(first.firstChild).toBe(node);
		expect(Array.from(highlights.get('syntax-str')!, (range) => range.toString())).toEqual([
			'😀 x',
			'\t你好',
			'😀 x',
			'\t你好'
		]);
		clearFirst();
		expect(highlights.get('syntax-str')?.size).toBe(2);
		clearSecond();
		expect(highlights.size).toBe(0);
	});

	it('keeps source readable without the browser highlight API', () => {
		vi.stubGlobal('CSS', {});
		const element = document.createElement('pre');
		element.textContent = '<script>alert("text")</script>';
		paintTokens(element, [{ start: 0, end: 6, type: 'kwd' }])();
		expect(element.textContent).toBe('<script>alert("text")</script>');
		expect(element.querySelector('script')).toBeNull();
	});
});
