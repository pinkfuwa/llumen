import { describe, test, expect } from 'vitest';
import { parse } from './parser';
import { TokenType } from './tokens';
import type {
	HeadingToken,
	ParagraphToken,
	CodeBlockToken,
	BlockquoteToken,
	OrderedListToken,
	UnorderedListToken,
	TableToken,
	LatexBlockToken,
	LatexInlineToken,
	BoldToken,
	ItalicToken,
	StrikethroughToken,
	LinkToken,
	ImageToken,
	TextToken
} from './tokens';

// ============================================================================
// INCREMENTAL PARSING TESTS - One test per token type
// ============================================================================

describe('MarkdownParser - Incremental Parsing: Individual Token Types', () => {
	test('incremental: heading token', () => {
		const initial = '# Hello';
		const appended = '\n## World';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(2);
		expect(result.tokens[0].type).toBe(TokenType.Heading);
		expect((result.tokens[0] as HeadingToken).level).toBe(1);
		expect(result.tokens[1].type).toBe(TokenType.Heading);
		expect((result.tokens[1] as HeadingToken).level).toBe(2);
	});

	test('incremental: paragraph token', () => {
		const initial = 'First paragraph.';
		const appended = '\n\nSecond paragraph.';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(2);
		expect(result.tokens[0].type).toBe(TokenType.Paragraph);
		expect(result.tokens[1].type).toBe(TokenType.Paragraph);
	});

	test('incremental: code block token', () => {
		const initial = '```js\nconst x = 1;\n```';
		const appended = '\n\n```py\nprint("hi")\n```';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(2);
		expect(result.tokens[0].type).toBe(TokenType.CodeBlock);
		expect((result.tokens[0] as CodeBlockToken).language).toBe('js');
		expect(result.tokens[1].type).toBe(TokenType.CodeBlock);
		expect((result.tokens[1] as CodeBlockToken).language).toBe('py');
	});

	test('incremental: blockquote token', () => {
		const initial = '> First quote\n\n';
		const appended = '> Second quote';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(2);
		expect(result.tokens[0].type).toBe(TokenType.Blockquote);
		expect(result.tokens[1].type).toBe(TokenType.Blockquote);
	});

	test('incremental: ordered list token', () => {
		const initial = '1. First list\n2. Item two\n\n';
		const appended = '1. Second list\n2. Another item';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(2);
		expect(result.tokens[0].type).toBe(TokenType.OrderedList);
		expect(result.tokens[1].type).toBe(TokenType.OrderedList);
	});

	test('incremental: unordered list token', () => {
		const initial = '- First list\n- Item two\n\n';
		const appended = '- Second list\n- Another item';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(2);
		expect(result.tokens[0].type).toBe(TokenType.UnorderedList);
		expect(result.tokens[1].type).toBe(TokenType.UnorderedList);
	});

	test('incremental: table token', () => {
		const initial = '| A | B |\n|---|---|\n| 1 | 2 |\n\n';
		const appended = '| X | Y |\n|---|---|\n| 3 | 4 |';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(2);
		expect(result.tokens[0].type).toBe(TokenType.Table);
		expect(result.tokens[1].type).toBe(TokenType.Table);
	});

	test('incremental: horizontal rule token', () => {
		const initial = '---';
		const appended = '\n\n***';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(2);
		expect(result.tokens[0].type).toBe(TokenType.HorizontalRule);
		expect(result.tokens[1].type).toBe(TokenType.HorizontalRule);
	});

	test('incremental: latex block token', () => {
		const initial = '$$\nx^2\n$$';
		const appended = '\n\n$$\ny^3\n$$';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(2);
		expect(result.tokens[0].type).toBe(TokenType.LatexBlock);
		expect(result.tokens[1].type).toBe(TokenType.LatexBlock);
	});

	test('incremental: latex inline token', () => {
		const initial = 'Inline $x^2$ math';
		const appended = ' and $y^3$ more';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(1);
		const para = result.tokens[0] as ParagraphToken;
		const latexTokens = para.children?.filter((t) => t.type === TokenType.LatexInline) || [];
		expect(latexTokens.length).toBe(2);
	});

	test('incremental: bold token', () => {
		const initial = '**Bold one**';
		const appended = ' and **bold two**';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(1);
		const para = result.tokens[0] as ParagraphToken;
		const boldTokens = para.children?.filter((t) => t.type === TokenType.Bold) || [];
		expect(boldTokens.length).toBe(2);
	});

	test('incremental: italic token', () => {
		const initial = '*Italic one*';
		const appended = ' and *italic two*';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(1);
		const para = result.tokens[0] as ParagraphToken;
		const italicTokens = para.children?.filter((t) => t.type === TokenType.Italic) || [];
		expect(italicTokens.length).toBe(2);
	});

	test('incremental: strikethrough token', () => {
		const initial = '~~Strike one~~';
		const appended = ' and ~~strike two~~';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(1);
		const para = result.tokens[0] as ParagraphToken;
		const strikeTokens = para.children?.filter((t) => t.type === TokenType.Strikethrough) || [];
		expect(strikeTokens.length).toBe(2);
	});

	test('incremental: inline code token', () => {
		const initial = '`code1`';
		const appended = ' and `code2`';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(1);
		const para = result.tokens[0] as ParagraphToken;
		const codeTokens = para.children?.filter((t) => t.type === TokenType.InlineCode) || [];
		expect(codeTokens.length).toBe(2);
	});

	test('incremental: link token', () => {
		const initial = '[Link 1](http://one.com)';
		const appended = ' and [Link 2](http://two.com)';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(1);
		const para = result.tokens[0] as ParagraphToken;
		const linkTokens = para.children?.filter((t) => t.type === TokenType.Link) || [];
		expect(linkTokens.length).toBe(2);
	});

	test('incremental: image token', () => {
		const initial = '![Alt 1](http://one.jpg)';
		const appended = ' and ![Alt 2](http://two.jpg)';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(1);
		const para = result.tokens[0] as ParagraphToken;
		const imageTokens = para.children?.filter((t) => t.type === TokenType.Image) || [];
		expect(imageTokens.length).toBe(2);
	});

	test('incremental: line break token', () => {
		const initial = 'Line one  \n';
		const appended = 'Line two  \nLine three';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(1);
		const para = result.tokens[0] as ParagraphToken;
		const brTokens = para.children?.filter((t) => t.type === TokenType.LineBreak) || [];
		expect(brTokens.length).toBe(2);
	});
});

// ============================================================================
// INCREMENTAL PARSING TESTS - Mixed tokens (3 tests)
// ============================================================================

describe('MarkdownParser - Incremental Parsing: Mixed Tokens', () => {
	test('mixed: heading + paragraph + code block', () => {
		const initial = '# Title\n\nSome text.';
		const appended = '\n\n```js\ncode\n```';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(3);
		expect(result.tokens[0].type).toBe(TokenType.Heading);
		expect(result.tokens[1].type).toBe(TokenType.Paragraph);
		expect(result.tokens[2].type).toBe(TokenType.CodeBlock);
	});

	test('mixed: list + table + blockquote', () => {
		const initial = '- Item 1\n- Item 2';
		const appended = '\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\n> Quote';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(3);
		expect(result.tokens[0].type).toBe(TokenType.UnorderedList);
		expect(result.tokens[1].type).toBe(TokenType.Table);
		expect(result.tokens[2].type).toBe(TokenType.Blockquote);
	});

	test('mixed: paragraph with inline formatting + latex', () => {
		const initial = 'Text with **bold** and *italic*.';
		const appended = ' Also $x^2$ reference.';
		const combined = initial + appended;

		const result = parse(combined);
		expect(result.tokens.length).toBe(1);
		const para = result.tokens[0] as ParagraphToken;
		expect(para.children?.some((t) => t.type === TokenType.Bold)).toBe(true);
		expect(para.children?.some((t) => t.type === TokenType.Italic)).toBe(true);
		expect(para.children?.some((t) => t.type === TokenType.LatexInline)).toBe(true);
	});
});

// ============================================================================
// TABLE WITH TABS (SPACES) TESTS
// ============================================================================

describe('MarkdownParser - Table with Tabs/Spaces', () => {
	test('table with tab-separated columns', () => {
		const markdown = 'A\tB\tC\n---\t---\t---\n1\t2\t3';
		const result = parse(markdown);

		expect(result.tokens.length).toBe(1);
		const table = result.tokens[0] as TableToken;
		expect(table.type).toBe(TokenType.Table);
		expect(table.children?.length).toBeGreaterThan(0);
	});

	test('table with mixed tabs and spaces', () => {
		const markdown = 'Col1\t  Col2  \tCol3\n---\t---\t---\nVal1\t  Val2  \tVal3';
		const result = parse(markdown);

		expect(result.tokens.length).toBe(1);
		const table = result.tokens[0] as TableToken;
		expect(table.type).toBe(TokenType.Table);
	});

	test('table with only spaces (standard pipe-delimited)', () => {
		const markdown = '| A   | B   | C   |\n|-----|-----|-----|\n| 1   | 2   | 3   |';
		const result = parse(markdown);

		expect(result.tokens.length).toBe(1);
		const table = result.tokens[0] as TableToken;
		expect(table.type).toBe(TokenType.Table);
		expect(table.children?.length).toBeGreaterThan(0);
	});

	test('table with tabs creates region boundary', () => {
		const markdown = 'A\tB\n---\t---\n1\t2';
		const result = parse(markdown);

		const tableRegion = result.regions.find((r) => r.type === 'table');
		expect(tableRegion).toBeDefined();
		expect(tableRegion?.start).toBeGreaterThanOrEqual(0);
		expect(tableRegion?.end).toBeGreaterThan(tableRegion!.start);
	});
});


// ============================================================================
// ORIGINAL COMPREHENSIVE TESTS (from previous implementation)
// ============================================================================

describe('MarkdownParser - Headings', () => {
	test('parses h1 heading', () => {
		const result = parse('# Hello World');

		const heading = result.tokens[0] as HeadingToken;
		expect(heading.type).toBe(TokenType.Heading);
		expect(heading.level).toBe(1);
	});

	test('parses h6 heading', () => {
		const result = parse('###### Level 6');
		const heading = result.tokens[0] as HeadingToken;
		expect(heading.type).toBe(TokenType.Heading);
		expect(heading.level).toBe(6);
	});

	test('parses heading with inline formatting', () => {
		const result = parse('# Hello **Bold** World');
		const heading = result.tokens[0] as HeadingToken;
		expect(heading.children?.some((t) => t.type === TokenType.Bold)).toBe(true);
	});

	test('does not parse heading without space', () => {
		const result = parse('#NoSpace');
		expect(result.tokens[0].type).toBe(TokenType.Paragraph);
	});
});

describe('MarkdownParser - Paragraphs', () => {
	test('parses simple paragraph', () => {
		const result = parse('This is a paragraph.');

		const para = result.tokens[0] as ParagraphToken;
		expect(para.type).toBe(TokenType.Paragraph);
	});

	test('parses multi-line paragraph', () => {
		const result = parse('Line one\nLine two');
		expect(result.tokens.length).toBe(1);
	});

	test('splits paragraphs on blank lines', () => {
		const result = parse('First paragraph.\n\nSecond paragraph.');
		expect(result.tokens.length).toBe(2);
	});
});

describe('MarkdownParser - Code Blocks', () => {
	test('parses fenced code block without language', () => {
		const markdown = '```\ncode here\n```';
		const result = parse(markdown);
		const code = result.tokens[0] as CodeBlockToken;
		expect(code.type).toBe(TokenType.CodeBlock);
		expect(code.language).toBeUndefined();
	});

	test('parses fenced code block with language', () => {
		const markdown = '```javascript\nconst x = 1;\n```';
		const result = parse(markdown);
		const code = result.tokens[0] as CodeBlockToken;
		expect(code.type).toBe(TokenType.CodeBlock);
		expect(code.language).toBe('javascript');
	});

	test('code block creates region boundary', () => {
		const markdown = '```\ncode\n```';
		const result = parse(markdown);
		expect(result.regions.some((r) => r.type === 'codeblock')).toBe(true);
	});
});

describe('MarkdownParser - Blockquotes', () => {
	test('parses simple blockquote', () => {
		const markdown = '> This is a quote';
		const result = parse(markdown);
		const quote = result.tokens[0] as BlockquoteToken;
		expect(quote.type).toBe(TokenType.Blockquote);
	});

	test('parses multi-line blockquote', () => {
		const markdown = '> Line 1\n> Line 2';
		const result = parse(markdown);
		expect(result.tokens.length).toBe(1);
	});

	test('blockquote creates region boundary', () => {
		const markdown = '> Quote';
		const result = parse(markdown);
		expect(result.regions.some((r) => r.type === 'blockquote')).toBe(true);
	});
});

describe('MarkdownParser - Lists', () => {
	test('parses unordered list with dash', () => {
		const markdown = '- Item 1\n- Item 2';
		const result = parse(markdown);
		const list = result.tokens[0] as UnorderedListToken;
		expect(list.type).toBe(TokenType.UnorderedList);
	});

	test('parses unordered list with asterisk', () => {
		const markdown = '* Item 1\n* Item 2';
		const result = parse(markdown);
		expect(result.tokens[0].type).toBe(TokenType.UnorderedList);
	});

	test('parses ordered list', () => {
		const markdown = '1. First\n2. Second';
		const result = parse(markdown);
		const list = result.tokens[0] as OrderedListToken;
		expect(list.type).toBe(TokenType.OrderedList);
		expect(list.startNumber).toBe(1);
	});

	test('parses ordered list starting from non-1', () => {
		const markdown = '5. Fifth\n6. Sixth';
		const result = parse(markdown);
		const list = result.tokens[0] as OrderedListToken;
		expect(list.startNumber).toBe(5);
	});

	test('list creates region boundary', () => {
		const markdown = '- Item';
		const result = parse(markdown);
		expect(result.regions.some((r) => r.type === 'list')).toBe(true);
	});
});

describe('MarkdownParser - Tables', () => {
	test('parses table with pipes', () => {
		const markdown = '| A | B |\n|---|---|\n| 1 | 2 |';
		const result = parse(markdown);
		const table = result.tokens[0] as TableToken;
		expect(table.type).toBe(TokenType.Table);
	});

	test('parses table with tabs (non-standard)', () => {
		const markdown = 'A\tB\n---\t---\n1\t2';
		const result = parse(markdown);
		const table = result.tokens[0] as TableToken;
		expect(table.type).toBe(TokenType.Table);
	});

	test('table creates region boundary', () => {
		const markdown = '| A | B |\n|---|---|\n| 1 | 2 |';
		const result = parse(markdown);
		expect(result.regions.some((r) => r.type === 'table')).toBe(true);
	});

	test('table with multiple rows', () => {
		const markdown = '| A | B |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |';
		const result = parse(markdown);
		const table = result.tokens[0] as TableToken;
		expect(table.children?.length).toBeGreaterThan(2);
	});

	test('parses table with Chinese characters and preceding text', () => {
		const markdown =
			'效果：\n| 欄位1 | 欄位2 | 欄位3 |\n|-------|-------|-------|\n| 內容1 | 內容2 | 內容3 |\n| 內容4 | 內容5 | 內容6 |';
		const result = parse(markdown);
		expect(result.tokens.length).toBe(2);

		const para = result.tokens[0] as ParagraphToken;
		expect(para.type).toBe(TokenType.Paragraph);
		const textToken = para.children?.[0] as TextToken;
		expect(textToken.type).toBe(TokenType.Text);
		expect(textToken.content).toBe('效果：');

		const table = result.tokens[1] as TableToken;
		expect(table.type).toBe(TokenType.Table);
		expect(table.children?.length).toBeGreaterThan(2);
	});
});

describe('MarkdownParser - Horizontal Rules', () => {
	test('parses horizontal rule with dashes', () => {
		const markdown = '---';
		const result = parse(markdown);
		expect(result.tokens[0].type).toBe(TokenType.HorizontalRule);
	});

	test('parses horizontal rule with asterisks', () => {
		const markdown = '***';
		const result = parse(markdown);
		expect(result.tokens[0].type).toBe(TokenType.HorizontalRule);
	});

	test('parses horizontal rule with underscores', () => {
		const markdown = '___';
		const result = parse(markdown);
		expect(result.tokens[0].type).toBe(TokenType.HorizontalRule);
	});
});

describe('MarkdownParser - LaTeX Block', () => {
	test('parses LaTeX block with \\[ \\]', () => {
		const markdown = '\\[\nx^2\n\\]';
		const result = parse(markdown);
		const latex = result.tokens[0] as LatexBlockToken;
		expect(latex.type).toBe(TokenType.LatexBlock);
	});

	test('parses LaTeX block with $$ $$', () => {
		const markdown = '$$\ny^2\n$$';
		const result = parse(markdown);
		const latex = result.tokens[0] as LatexBlockToken;
		expect(latex.type).toBe(TokenType.LatexBlock);
	});

	test('does not parse $$ as block without newline', () => {
		const markdown = '$$ inline $$';
		const result = parse(markdown);
		expect(result.tokens[0].type).toBe(TokenType.Paragraph);
	});
});

describe('MarkdownParser - LaTeX Inline', () => {
	test('parses inline LaTeX with \\( \\)', () => {
		const markdown = 'Text \\(x^2\\) more';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline) as
			| LatexInlineToken
			| undefined;
		expect(latexToken).toBeDefined();
	});

	test('parses inline LaTeX with $ $ without spaces (non-standard)', () => {
		const markdown = 'Text $x^2$ more';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline) as
			| LatexInlineToken
			| undefined;
		expect(latexToken).toBeDefined();
	});

	test('parses inline LaTeX with $ $ without spaces for expressions (non-standard)', () => {
		const markdown = 'Equation $x^2+y^2$ works';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline) as
			| LatexInlineToken
			| undefined;
		expect(latexToken).toBeDefined();
	});

	test('requires space before $ for latex with spaces (non-standard)', () => {
		const markdown = 'Price $100 dollars';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline);
		expect(latexToken).toBeUndefined();
	});

	test('does not require space after opening $ (non-standard)', () => {
		const markdown = 'Text $x + y$ more';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline) as
			| LatexInlineToken
			| undefined;
		expect(latexToken).toBeDefined();
	});

	test('does not parse $ with leading space', () => {
		const markdown = 'Price $100';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const latexToken = para.children?.find((t) => t.type === TokenType.LatexInline);
		expect(latexToken).toBeUndefined();
	});
});

describe('MarkdownParser - Inline Formatting', () => {
	test('parses bold with **', () => {
		const markdown = 'Text **bold** here';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const boldToken = para.children?.find((t) => t.type === TokenType.Bold);
		expect(boldToken).toBeDefined();
	});

	test('parses bold with __', () => {
		const markdown = 'Text __bold__ here';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const boldToken = para.children?.find((t) => t.type === TokenType.Bold);
		expect(boldToken).toBeDefined();
	});

	test('parses italic with *', () => {
		const markdown = 'Text *italic* here';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const italicToken = para.children?.find((t) => t.type === TokenType.Italic);
		expect(italicToken).toBeDefined();
	});

	test('parses italic with _', () => {
		const markdown = 'Text _italic_ here';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const italicToken = para.children?.find((t) => t.type === TokenType.Italic);
		expect(italicToken).toBeDefined();
	});

	test('parses strikethrough with ~~', () => {
		const markdown = 'Text ~~strike~~ here';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const strikeToken = para.children?.find((t) => t.type === TokenType.Strikethrough) as
			| StrikethroughToken
			| undefined;
		expect(strikeToken).toBeDefined();
	});

	test('parses nested formatting', () => {
		const markdown = '**bold with *italic* inside**';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const boldToken = para.children?.find((t) => t.type === TokenType.Bold) as BoldToken;
		expect(boldToken).toBeDefined();
		const italicToken = boldToken?.children?.find((t) => t.type === TokenType.Italic);
		expect(italicToken).toBeDefined();
	});

	test('parses Chinese text with mixed inline formatting', () => {
		const markdown = '結果：*斜體文字*、**粗體文字**、~~刪除線~~';
		const result = parse(markdown);
		expect(result.tokens.length).toBe(1);
		const para = result.tokens[0] as ParagraphToken;
		expect(para.type).toBe(TokenType.Paragraph);
		expect(para.children).toBeDefined();

		// Verify tokens are in correct order with proper content
		expect(para.children?.length).toBeGreaterThan(0);

		// Check that first token is text "結果："
		const firstChild = para.children?.[0] as TextToken;
		expect(firstChild.type).toBe(TokenType.Text);
		expect(firstChild.content).toBe('結果：');

		// Check italic token exists with correct content
		const italicToken = para.children?.find((t) => t.type === TokenType.Italic) as
			| ItalicToken
			| undefined;
		expect(italicToken).toBeDefined();
		const italicContent = italicToken?.children?.[0] as TextToken | undefined;
		expect(italicContent?.content).toBe('斜體文字');

		// Check bold token exists with correct content
		const boldToken = para.children?.find((t) => t.type === TokenType.Bold) as
			| BoldToken
			| undefined;
		expect(boldToken).toBeDefined();
		const boldContent = boldToken?.children?.[0] as TextToken | undefined;
		expect(boldContent?.content).toBe('粗體文字');

		// Check strikethrough token exists with correct content
		const strikeToken = para.children?.find((t) => t.type === TokenType.Strikethrough) as
			| StrikethroughToken
			| undefined;
		expect(strikeToken).toBeDefined();
		const strikeContent = strikeToken?.children?.[0] as TextToken | undefined;
		expect(strikeContent?.content).toBe('刪除線');

		// Verify text tokens exist for punctuation between formatted elements
		const textTokens = para.children?.filter((t) => t.type === TokenType.Text) as
			| TextToken[]
			| undefined;
		expect(textTokens?.length).toBeGreaterThanOrEqual(3); // "結果：", "、", "、"
	});
});

describe('MarkdownParser - Inline Code', () => {
	test('parses inline code', () => {
		const markdown = 'Text `code` here';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const codeToken = para.children?.find((t) => t.type === TokenType.InlineCode);
		expect(codeToken).toBeDefined();
	});

	test('parses multiple inline code segments', () => {
		const markdown = '`code1` and `code2`';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const codeTokens = para.children?.filter((t) => t.type === TokenType.InlineCode) || [];
		expect(codeTokens.length).toBe(2);
	});
});

describe('MarkdownParser - Links', () => {
	test('parses simple link', () => {
		const markdown = '[Link Text](https://example.com)';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const linkToken = para.children?.find((t) => t.type === TokenType.Link) as LinkToken;
		expect(linkToken).toBeDefined();
		expect(linkToken?.url).toBe('https://example.com');
	});

	test('parses link with formatted text', () => {
		const markdown = '[**Bold Link**](https://example.com)';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const linkToken = para.children?.find((t) => t.type === TokenType.Link) as LinkToken;
		expect(linkToken?.children?.some((t) => t.type === TokenType.Bold)).toBe(true);
	});
});

describe('MarkdownParser - Images', () => {
	test('parses image', () => {
		const markdown = '![Alt Text](https://example.com/image.png)';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const imageToken = para.children?.find((t) => t.type === TokenType.Image) as ImageToken;
		expect(imageToken).toBeDefined();
		expect(imageToken?.url).toBe('https://example.com/image.png');
	});

	test('parses image with empty alt text', () => {
		const markdown = '![](https://example.com/image.png)';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const imageToken = para.children?.find((t) => t.type === TokenType.Image) as ImageToken;
		expect(imageToken).toBeDefined();
	});
});

describe('MarkdownParser - Text Tokens', () => {
	test('creates text token for plain text', () => {
		const markdown = 'Just plain text';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		expect(para.children?.some((t) => t.type === TokenType.Text)).toBe(true);
	});

	test('merges consecutive text segments', () => {
		const markdown = 'Text without formatting';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		const textTokens = para.children?.filter((t) => t.type === TokenType.Text) || [];
		expect(textTokens.length).toBeGreaterThan(0);
	});
});

describe('MarkdownParser - Complex Documents', () => {
	test('parses document with multiple block types', () => {
		const markdown = `# Heading

Paragraph with **bold**.

- List item 1
- List item 2

\`\`\`
code
\`\`\`

> Quote`;

		const result = parse(markdown);
		expect(result.tokens.length).toBeGreaterThan(3);
	});

	test('parses inline elements within paragraphs', () => {
		const markdown = 'Text **bold** *italic* `code` [link](url) ![img](url)';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		expect(para.children?.length).toBeGreaterThan(5);
	});
});

describe('MarkdownParser - Region Boundaries', () => {
	test('tracks regions for incremental-like parsing', () => {
		const markdown = `> Quote

\`\`\`
code
\`\`\`

- List
- Items

| A | B |
|---|---|
| 1 | 2 |`;

		const result = parse(markdown);
		expect(result.regions.some((r) => r.type === 'blockquote')).toBe(true);
		expect(result.regions.some((r) => r.type === 'codeblock')).toBe(true);
		expect(result.regions.some((r) => r.type === 'list')).toBe(true);
		expect(result.regions.some((r) => r.type === 'table')).toBe(true);
	});

	test('region boundaries have correct positions', () => {
		const markdown = '```\ncode\n```';
		const result = parse(markdown);
		expect(result.regions[0].start).toBeGreaterThanOrEqual(0);
		expect(result.regions[0].end).toBeGreaterThan(result.regions[0].start);
	});
});

describe('MarkdownParser - Edge Cases', () => {
	test('handles empty string', () => {
		const result = parse('');
		expect(result.tokens.length).toBe(0);
	});

	test('handles whitespace only', () => {
		const result = parse('   \n  \n   ');
		expect(result.tokens.length).toBe(0);
	});

	test('handles unclosed bold', () => {
		const result = parse('**unclosed');
		expect(result.tokens.length).toBe(1);
	});

	test('handles unclosed code block', () => {
		const result = parse('```\nunclosed');
		expect(result.tokens.length).toBe(1);
		const codeBlock = result.tokens[0] as CodeBlockToken;
		expect(codeBlock.type).toBe(TokenType.CodeBlock);
		expect(codeBlock.content).toBe('unclosed');
		expect(codeBlock.language).toBeUndefined();
	});

	test('handles unclosed code block with language', () => {
		const result = parse('```javascript\nconst x = 1;\nconsole.log(x);');
		expect(result.tokens.length).toBe(1);
		const codeBlock = result.tokens[0] as CodeBlockToken;
		expect(codeBlock.type).toBe(TokenType.CodeBlock);
		expect(codeBlock.language).toBe('javascript');
		expect(codeBlock.content).toBe('const x = 1;\nconsole.log(x);');
	});

	test('streaming: code block displays immediately without closing delimiter', () => {
		// Simulate streaming: code block starts but hasn't received closing ``` yet
		const stream1 = '```python\ndef hello():';
		const result1 = parse(stream1);
		expect(result1.tokens.length).toBe(1);
		const codeBlock1 = result1.tokens[0] as CodeBlockToken;
		expect(codeBlock1.type).toBe(TokenType.CodeBlock);
		expect(codeBlock1.language).toBe('python');
		expect(codeBlock1.content).toBe('def hello():');

		// More content arrives
		const stream2 = '```python\ndef hello():\n    print("world")';
		const result2 = parse(stream2);
		expect(result2.tokens.length).toBe(1);
		const codeBlock2 = result2.tokens[0] as CodeBlockToken;
		expect(codeBlock2.type).toBe(TokenType.CodeBlock);
		expect(codeBlock2.language).toBe('python');
		expect(codeBlock2.content).toBe('def hello():\n    print("world")');

		// Final closing delimiter arrives
		const stream3 = '```python\ndef hello():\n    print("world")\n```';
		const result3 = parse(stream3);
		expect(result3.tokens.length).toBe(1);
		const codeBlock3 = result3.tokens[0] as CodeBlockToken;
		expect(codeBlock3.type).toBe(TokenType.CodeBlock);
		expect(codeBlock3.language).toBe('python');
		expect(codeBlock3.content).toBe('def hello():\n    print("world")');
	});

	test('handles special characters in text', () => {
		const markdown = 'Text with & < > " \' characters';
		const result = parse(markdown);
		expect(result.tokens.length).toBe(1);
	});
});

describe('MarkdownParser - Position Tracking', () => {
	test('tracks start and end positions for blocks', () => {
		const markdown = '# Heading\n\nParagraph';
		const result = parse(markdown);
		expect(result.tokens[0].start).toBe(0);
		expect(result.tokens[0].end).toBeGreaterThan(0);
	});

	test('tracks positions for inline elements', () => {
		const markdown = 'Text **bold** text';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		expect(para.children?.every((t) => t.start >= 0 && t.end > t.start)).toBe(true);
	});
});

describe('MarkdownParser - List with indented code block and trailing text', () => {
	test('parses list with indented code block followed by text as separate paragraph', () => {
		const markdown = `*   **Actionable Step:** If you are using C++ or Fortran, just use an OpenMP pragma:
    \`\`\`cpp
    #pragma omp parallel for num_threads(4)
    for (int i = 0; i < N; i++) {
        for (int j = 0; j < N; j++) {
            for (int k = 0; k < N; k++) {
                C[i][j] += A[i][k] * B[k][j];
            }
        }
    }
    \`\`\`
AAA`;

		const result = parse(markdown);

		// Should have a list and a paragraph
		expect(result.tokens.length).toBeGreaterThanOrEqual(2);

		// First token should be a list
		expect(result.tokens[0].type).toBe(TokenType.UnorderedList);

		// Find the text token "AAA" - it should be in a separate paragraph, not in the code block
		const lastToken = result.tokens[result.tokens.length - 1];
		expect(lastToken.type).toBe(TokenType.Paragraph);

		const paraToken = lastToken as ParagraphToken;
		const textContent = paraToken.children
			?.filter((t) => t.type === TokenType.Text)
			.map((t) => (t as TextToken).content)
			.join('');

		expect(textContent).toContain('AAA');
	});
});
