import { describe, expect, it } from 'vitest';
import { codeLanguage } from './languages';
import { tokenizeCode } from './tokenize';

describe('syntax token ranges', () => {
	it.each([
		['javascript', 'js'],
		[' TypeScript ', 'ts'],
		['python', 'py'],
		['rust', 'rs'],
		['dockerfile', 'docker'],
		['shellscript', 'bash'],
		['yml', 'yaml'],
		['text', 'plain']
	])('maps %s to %s', (alias, language) => {
		expect(codeLanguage(alias)).toBe(language);
	});

	it.each(['csharp', 'cpp', 'ruby', 'svelte', 'tsx', '__proto__', 'constructor'])(
		'leaves %s plain',
		(name) => {
			expect(codeLanguage(name)).toBeUndefined();
		}
	);

	it('uses UTF-16 offsets for Unicode, tabs, CRLF, and incomplete multiline input', () => {
		const source = 'const greeting = "你好 café 😀";\r\n\t/* unfinished\ncomment';
		for (let length = 0; length <= source.length; length++) {
			const prefix = source.slice(0, length);
			const tokens = tokenizeCode(prefix, 'js');
			let end = 0;
			for (const token of tokens) {
				expect(token.start).toBeGreaterThanOrEqual(end);
				expect(token.end).toBeGreaterThan(token.start);
				expect(token.end).toBeLessThanOrEqual(prefix.length);
				end = token.end;
			}
		}
		const string = tokenizeCode(source, 'js').find((token) => token.type === 'str');
		expect(string && source.slice(string.start, string.end)).toBe('"你好 café 😀"');
	});

	it('keeps multiline TOML strings in the same full-block tokenization', () => {
		const source = '[server]\nvalue = """first\nsecond"""\nenabled = true\n';
		const tokens = tokenizeCode(source, 'toml');
		expect(
			tokens.some(
				(token) => token.type === 'str' && source.slice(token.start, token.end).includes('\n')
			)
		).toBe(true);
		expect(tokens.some((token) => token.type === 'bool')).toBe(true);
	});

	it('resolves nested JavaScript and CSS in HTML from the same registry', () => {
		const source = '<script>const answer = 42;</script><style>body { color: red; }</style>';
		const tokens = tokenizeCode(source, 'html');
		expect(
			tokens.some((token) => token.type === 'num' && source.slice(token.start, token.end) === '42')
		).toBe(true);
		expect(
			tokens.some(
				(token) => token.type === 'kwd' && source.slice(token.start, token.end) === 'const'
			)
		).toBe(true);
	});
});
