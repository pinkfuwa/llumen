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
		clearFirst.clear();
		expect(highlights.get('syntax-str')?.size).toBe(2);
		clearSecond.clear();
		expect(highlights.size).toBe(0);
	});

	it('keeps source readable without the browser highlight API', () => {
		vi.stubGlobal('CSS', {});
		const element = document.createElement('pre');
		element.textContent = '<script>alert("text")</script>';
		paintTokens(element, [{ start: 0, end: 6, type: 'kwd' }]).clear();
		expect(element.textContent).toBe('<script>alert("text")</script>');
		expect(element.querySelector('script')).toBeNull();
	});

	it('restores collapsed live ranges on the same text node without coloring the appended suffix', () => {
		const highlights = new Map<string, Set<Range>>();
		vi.stubGlobal('CSS', { highlights });
		vi.stubGlobal('Highlight', Set);
		const element = document.createElement('pre');
		element.innerHTML = '<span data-code-offset="0">const value</span>';
		document.body.append(element);
		const paint = paintTokens(element, [{ start: 0, end: 5, type: 'kwd' }]);
		const range = Array.from(highlights.get('syntax-kwd')!)[0];
		const node = element.firstChild!.firstChild as Text;
		node.data += ' = 42;';
		expect(range.toString()).toBe('');
		paint.refresh();
		expect(range.toString()).toBe('const');
		expect(highlights.get('syntax-kwd')?.has(range)).toBe(true);
		expect(range.endOffset).toBe(5);
		expect(node.data).toBe('const value = 42;');
	});
});
