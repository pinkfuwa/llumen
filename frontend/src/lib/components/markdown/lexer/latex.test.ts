import { describe, test, expect } from 'vitest';
import { parse } from './index';
import { TokenType } from './tokens';
import type { ParagraphToken, LatexBlockToken, LatexInlineToken, Token } from './tokens';

describe('MarkdownParser - LaTeX Block Delimiters', () => {
	test('parses \\[ \\] block delimiter', () => {
		const markdown = '\\[\nx^2 + y^2\n\\]';
		const result = parse(markdown);
		const latex = result.tokens[0] as LatexBlockToken;
		expect(latex.type).toBe(TokenType.LatexBlock);
		expect(latex.content).toBe('x^2 + y^2');
	});

	test('parses \\[ \\] on same line (no newline required)', () => {
		const markdown = '\\[x^2\\]';
		const result = parse(markdown);
		const latex = result.tokens[0] as LatexBlockToken;
		expect(latex.type).toBe(TokenType.LatexBlock);
		expect(latex.content).toBe('x^2');
	});

	test('parses $$ $$ block delimiter with newline', () => {
		const markdown = '$$\ny^2 + x^2\n$$';
		const result = parse(markdown);
		const latex = result.tokens[0] as LatexBlockToken;
		expect(latex.type).toBe(TokenType.LatexBlock);
		expect(latex.content).toBe('y^2 + x^2');
	});

	test('$$ requires newline after opening delimiter', () => {
		const markdown = '$$ inline $$';
		const result = parse(markdown);
		expect(result.tokens[0].type).toBe(TokenType.Paragraph);
	});

	test('$$ with space instead of newline is not block', () => {
		const markdown = '$$ x^2 $$';
		const result = parse(markdown);
		expect(result.tokens[0].type).toBe(TokenType.Paragraph);
	});

	test('unclosed \\[ is not parsed as latex', () => {
		const markdown = '\\[ x^2';
		const result = parse(markdown);
		expect(result.tokens[0].type).toBe(TokenType.Paragraph);
	});

	test('unclosed $$ is not parsed as latex', () => {
		const markdown = '$$\nx^2';
		const result = parse(markdown);
		expect(result.tokens[0].type).toBe(TokenType.Paragraph);
	});
});

describe('MarkdownParser - LaTeX Inline Delimiters', () => {
	test('parses \\( \\) inline delimiter', () => {
		const markdown = 'Text \\(x^2\\) more';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline) as
			| LatexInlineToken
			| undefined;
		expect(latexToken).toBeDefined();
		expect(latexToken?.content).toBe('x^2');
	});

	test('\\( \\) works without surrounding spaces', () => {
		const markdown = 'word\\(x\\)word';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline);
		expect(latexToken).toBeDefined();
	});

	test('unclosed \\( is not parsed as latex', () => {
		const markdown = 'Text \\(x^2 more';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline);
		expect(latexToken).toBeUndefined();
	});
});

describe('MarkdownParser - LaTeX $ Delimiter Without Spaces Inside', () => {
	test('$x$ is valid latex (no spaces inside)', () => {
		const markdown = '$x$';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline) as
			| LatexInlineToken
			| undefined;
		expect(latexToken).toBeDefined();
		expect(latexToken?.content).toBe('x');
	});

	test('$x^2$ works without surrounding spaces', () => {
		const markdown = 'word$x^2$word';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline) as
			| LatexInlineToken
			| undefined;
		expect(latexToken).toBeDefined();
		expect(latexToken?.content).toBe('x^2');
	});

	test('$abc$ works in middle of sentence', () => {
		const markdown = 'The equation $E=mc^2$ is famous';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline) as
			| LatexInlineToken
			| undefined;
		expect(latexToken).toBeDefined();
		expect(latexToken?.content).toBe('E=mc^2');
	});

	test('$x$ at start of line', () => {
		const markdown = '$x$ is a variable';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline) as
			| LatexInlineToken
			| undefined;
		expect(latexToken).toBeDefined();
	});

	test('$x$ at end of line', () => {
		const markdown = 'The value is $x$';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline) as
			| LatexInlineToken
			| undefined;
		expect(latexToken).toBeDefined();
	});
});

describe('MarkdownParser - LaTeX $ Delimiter With Spaces Inside', () => {
	test('$ x + y $ requires space before and after', () => {
		const markdown = 'Text $ x + y $ more';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline) as
			| LatexInlineToken
			| undefined;
		expect(latexToken).toBeDefined();
		expect(latexToken?.content).toBe(' x + y ');
	});

	test('$ x + y $ at start with space after is valid', () => {
		const markdown = '$ x + y $ more';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline) as
			| LatexInlineToken
			| undefined;
		expect(latexToken).toBeDefined();
	});

	test('$ x + y $ at end is valid', () => {
		const markdown = 'Text $ x + y $';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline) as
			| LatexInlineToken
			| undefined;
		expect(latexToken).toBeDefined();
	});

	test('word$ x + y $ without space before is NOT valid', () => {
		const markdown = 'word$ x + y $ more';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline);
		expect(latexToken).toBeUndefined();
	});

	test('$ x + y $word without space after is NOT valid', () => {
		const markdown = 'Text $ x + y $word';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline);
		expect(latexToken).toBeUndefined();
	});

	test('I have $1 and $ is NOT valid latex', () => {
		const markdown = 'I have $1 and $';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline);
		expect(latexToken).toBeUndefined();
	});

	test('$1 and $5 with spaces inside are NOT valid', () => {
		const markdown = '$1 and $5';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexTokens = para.children?.filter((t) => t.type === TokenType.LatexInline);
		expect(latexTokens?.length).toBe(0);
	});
});

describe('MarkdownParser - LaTeX $ Edge Cases', () => {
	test('$$ is not inline latex (block delimiter)', () => {
		const markdown = 'Text $$ more';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline);
		expect(latexToken).toBeUndefined();
	});

	test('$ with space after opening is not valid', () => {
		const markdown = '$ x$';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline);
		expect(latexToken).toBeUndefined();
	});

	test('empty $$ is not valid', () => {
		const markdown = 'Text $$ more';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline);
		expect(latexToken).toBeUndefined();
	});

	test('unclosed $ is not latex', () => {
		const markdown = 'Text $x^2';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline);
		expect(latexToken).toBeUndefined();
	});

	test('price $100 is not latex', () => {
		const markdown = 'Price $100';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline);
		expect(latexToken).toBeUndefined();
	});

	test('I have $5 and you have $10 are not latex', () => {
		const markdown = 'I have $5 and you have $10';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexTokens = para.children?.filter((t) => t.type === TokenType.LatexInline);
		expect(latexTokens?.length).toBe(0);
	});
});

describe('MarkdownParser - LaTeX Multiple Delimiters', () => {
	test('multiple inline latex with $', () => {
		const markdown = 'First $x$ and second $y$ done';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexTokens = para.children?.filter(
			(t) => t.type === TokenType.LatexInline
		) as LatexInlineToken[];
		expect(latexTokens.length).toBe(2);
		expect(latexTokens[0].content).toBe('x');
		expect(latexTokens[1].content).toBe('y');
	});

	test('multiple inline latex with \\( \\)', () => {
		const markdown = 'First \\(a\\) and second \\(b\\) done';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexTokens = para.children?.filter(
			(t) => t.type === TokenType.LatexInline
		) as LatexInlineToken[];
		expect(latexTokens.length).toBe(2);
		expect(latexTokens[0].content).toBe('a');
		expect(latexTokens[1].content).toBe('b');
	});

	test('mixed delimiters in same paragraph', () => {
		const markdown = 'Use $x$ or \\(y\\) here';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexTokens = para.children?.filter((t) => t.type === TokenType.LatexInline);
		expect(latexTokens?.length).toBe(2);
	});
});

describe('MarkdownParser - LaTeX Content Edge Cases', () => {
	test('latex with complex expression', () => {
		const markdown = '$\\frac{-b \\pm \\sqrt{b^2-4ac}}{2a}$';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline) as
			| LatexInlineToken
			| undefined;
		expect(latexToken).toBeDefined();
		expect(latexToken?.content).toBe('\\frac{-b \\pm \\sqrt{b^2-4ac}}{2a}');
	});

	test('latex with underscores and subscripts', () => {
		const markdown = '$x_1 + x_2 + x_n$';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline) as
			| LatexInlineToken
			| undefined;
		expect(latexToken).toBeDefined();
	});

	test('block latex with multi-line content', () => {
		const markdown = '$$\n\\begin{align}\nx &= y \\\\\ny &= z\n\\end{align}\n$$';
		const result = parse(markdown);
		const latex = result.tokens[0] as LatexBlockToken;
		expect(latex.type).toBe(TokenType.LatexBlock);
		expect(latex.content).toContain('begin{align}');
	});
});
