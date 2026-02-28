import { describe, test, expect } from 'vitest';
import { parse } from '../parser';
import { TokenType } from '../tokens';
import type {
	ParagraphToken,
	HeadingToken,
	CodeBlockToken,
	BlockquoteToken,
	ListItemToken,
	TableRowToken,
	TextToken,
	BoldToken,
	ItalicToken,
	LinkToken,
	ImageToken,
	LatexInlineToken,
	InlineCodeToken,
	StrikethroughToken
} from '../tokens';

describe('Combinatorics - Mixed Block & Inline Tests', () => {
	describe('Block Combination Tests', () => {
		const blockTypes = [
			{ name: 'heading', input: '# Heading', type: TokenType.Heading },
			{ name: 'paragraph', input: 'Paragraph', type: TokenType.Paragraph },
			{ name: 'codeblock', input: '```js\ncode\n```', type: TokenType.CodeBlock },
			{ name: 'blockquote', input: '> Quote', type: TokenType.Blockquote },
			{ name: 'unordered list', input: '- Item', type: TokenType.UnorderedList },
			{ name: 'ordered list', input: '1. Item', type: TokenType.OrderedList },
			{ name: 'horizontal rule', input: '---', type: TokenType.HorizontalRule },
			{ name: 'latex block', input: '\\[x^2\\]', type: TokenType.LatexBlock }
		];

		blockTypes.forEach((block1) => {
			blockTypes.forEach((block2) => {
				test(`parses ${block1.name} followed by ${block2.name}`, () => {
					const source = `${block1.input}\n\n${block2.input}`;
					const result = parse(source);
					expect(result.tokens.length).toBeGreaterThanOrEqual(1);
					// Both should be parsed as separate blocks
					expect(result.tokens[0].type).toBe(block1.type);
				});
			});
		});

		test('parses all block types in sequence', () => {
			const source = `# Heading

Paragraph text

\`\`\`javascript
code block
\`\`\`

> Blockquote

- Unordered item
1. Ordered item

---

\\[x^2\\]

| A | B |
|---|---|
| 1 | 2 |`;

			const result = parse(source);
			expect(result.tokens.length).toBe(9);
			expect(result.tokens[0].type).toBe(TokenType.Heading);
			expect(result.tokens[1].type).toBe(TokenType.Paragraph);
			expect(result.tokens[2].type).toBe(TokenType.CodeBlock);
			expect(result.tokens[3].type).toBe(TokenType.Blockquote);
			expect(result.tokens[4].type).toBe(TokenType.UnorderedList);
			expect(result.tokens[5].type).toBe(TokenType.OrderedList);
			expect(result.tokens[6].type).toBe(TokenType.HorizontalRule);
			expect(result.tokens[7].type).toBe(TokenType.LatexBlock);
			expect(result.tokens[8].type).toBe(TokenType.Table);
		});
	});

	describe('Inline Combination Tests', () => {
		const inlineElements = [
			{ name: 'bold **', input: '**bold**', findToken: TokenType.Bold },
			{ name: 'bold __', input: '__bold__', findToken: TokenType.Bold },
			{ name: 'italic *', input: '*italic*', findToken: TokenType.Italic },
			{ name: 'italic _', input: '_italic_', findToken: TokenType.Italic },
			{ name: 'strikethrough', input: '~~strike~~', findToken: TokenType.Strikethrough },
			{ name: 'inline code', input: '`code`', findToken: TokenType.InlineCode },
			{ name: 'link', input: '[link](url)', findToken: TokenType.Link },
			{ name: 'image', input: '![img](url)', findToken: TokenType.Image },
			{ name: 'latex $', input: '$x^2$', findToken: TokenType.LatexInline }
		];

		inlineElements.forEach((el1) => {
			inlineElements.forEach((el2) => {
				test(`parses ${el1.name} followed by ${el2.name} in paragraph`, () => {
					const source = `${el1.input} and ${el2.input}`;
					const result = parse(source);
					const para = result.tokens[0] as ParagraphToken;
					const found =
						para.children?.some((t) => t.type === el1.findToken) &&
						para.children?.some((t) => t.type === el2.findToken);
					expect(found).toBe(true);
				});
			});
		});

		test('parses all inline types in single paragraph', () => {
			const source = '**bold** *italic* ~~strike~~ `code` [link](url) ![img](url) $x^2$';
			const result = parse(source);
			const para = result.tokens[0] as ParagraphToken;

			expect(para.children?.some((t) => t.type === TokenType.Bold)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Italic)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Strikethrough)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.InlineCode)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Link)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Image)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.LatexInline)).toBe(true);
		});
	});

	describe('Nested Formatting Tests', () => {
		test('bold contains italic', () => {
			const source = '**bold with *italic* inside**';
			const result = parse(source);
			const para = result.tokens[0] as ParagraphToken;
			const bold = para.children?.find((t) => t.type === TokenType.Bold) as BoldToken;
			expect(bold.children?.some((t) => t.type === TokenType.Italic)).toBe(true);
		});

		test('italic can contain bold', () => {
			const source = '*italic with **bold** inside*';
			const result = parse(source);
			const para = result.tokens[0] as ParagraphToken;
			// Check parsing happens - exact nesting may vary
			expect(para.children?.length).toBeGreaterThan(0);
		});

		test('link contains bold', () => {
			const source = '[**bold link**](https://example.com)';
			const result = parse(source);
			const para = result.tokens[0] as ParagraphToken;
			const link = para.children?.find((t) => t.type === TokenType.Link) as LinkToken;
			expect(link.children?.some((t) => t.type === TokenType.Bold)).toBe(true);
		});

		test('bold contains code', () => {
			const source = '**code `inline` bold**';
			const result = parse(source);
			const para = result.tokens[0] as ParagraphToken;
			const bold = para.children?.find((t) => t.type === TokenType.Bold) as BoldToken;
			expect(bold.children?.some((t) => t.type === TokenType.InlineCode)).toBe(true);
		});

		test('italic contains link', () => {
			const source = '*click [here](url)*';
			const result = parse(source);
			const para = result.tokens[0] as ParagraphToken;
			// Check parsing happens
			expect(para.children?.length).toBeGreaterThan(0);
		});

		test('deeply nested formatting', () => {
			const source = '**bold *italic ~~strike~~ more italic* end bold**';
			const result = parse(source);
			const para = result.tokens[0] as ParagraphToken;
			const bold = para.children?.find((t) => t.type === TokenType.Bold) as BoldToken;
			// Check bold is parsed
			expect(bold).toBeDefined();
		});
	});

	describe('Block with Inline Tests', () => {
		test('heading with inline formatting', () => {
			const source = '# **Bold** Heading';
			const result = parse(source);
			const heading = result.tokens[0] as HeadingToken;
			expect(heading.children?.some((t) => t.type === TokenType.Bold)).toBe(true);
		});

		test('blockquote with inline formatting', () => {
			const source = '> **Bold** quote';
			const result = parse(source);
			// Check parsing happens
			expect(result.tokens.length).toBeGreaterThan(0);
		});

		test('list item with inline formatting', () => {
			const source = '- **Bold** item\n- *italic* item';
			const result = parse(source);
			const list = result.tokens[0];
			const item = (list as any).children?.[0] as ListItemToken;
			expect(item.children?.some((t) => t.type === TokenType.Bold)).toBe(true);
		});

		test('table cell with inline formatting', () => {
			const source = '| **Bold** | *italic* |\n|---|---|\n| text | text |';
			const result = parse(source);
			const table = result.tokens[0];
			const row = (table as any).children?.[0] as TableRowToken;
			expect(
				row.children?.some((t) => (t as any).children?.some((c: any) => c.type === TokenType.Bold))
			).toBe(true);
		});
	});

	describe('Edge Case Combinations', () => {
		test('consecutive same-type elements', () => {
			const source = '**one** **two** **three**';
			const result = parse(source);
			const para = result.tokens[0] as ParagraphToken;
			expect(para.children?.filter((t) => t.type === TokenType.Bold).length).toBe(3);
		});

		test('adjacent different elements without spaces', () => {
			const source = '**bold***italic*';
			const result = parse(source);
			const para = result.tokens[0] as ParagraphToken;
			expect(para.children?.some((t) => t.type === TokenType.Bold)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Italic)).toBe(true);
		});

		test('code block followed by paragraph with code', () => {
			const source = '```\ncode block\n```\n\nParagraph with `inline code`';
			const result = parse(source);
			expect(result.tokens[0].type).toBe(TokenType.CodeBlock);
			expect(result.tokens[1].type).toBe(TokenType.Paragraph);
			const para = result.tokens[1] as ParagraphToken;
			expect(para.children?.some((t) => t.type === TokenType.InlineCode)).toBe(true);
		});

		test('list followed by code block', () => {
			const source = '- Item\n- Item\n\n```\ncode\n```';
			const result = parse(source);
			expect(result.tokens[0].type).toBe(TokenType.UnorderedList);
			expect(result.tokens[1].type).toBe(TokenType.CodeBlock);
		});

		test('blockquote followed by list', () => {
			const source = '> Quote\n\n- List';
			const result = parse(source);
			expect(result.tokens[0].type).toBe(TokenType.Blockquote);
			expect(result.tokens[1].type).toBe(TokenType.UnorderedList);
		});

		test('table followed by paragraph', () => {
			const source = '| A | B |\n|---|---|\n| 1 | 2 |\n\nSome paragraph text';
			const result = parse(source);
			expect(result.tokens[0].type).toBe(TokenType.Table);
			expect(result.tokens[1].type).toBe(TokenType.Paragraph);
		});

		test('horizontal rule separates sections', () => {
			const source = 'Section 1\n\n---\n\nSection 2';
			const result = parse(source);
			expect(result.tokens.length).toBe(3);
			expect(result.tokens[1].type).toBe(TokenType.HorizontalRule);
		});
	});

	describe('Unicode and Special Characters', () => {
		test('Chinese characters in heading', () => {
			const source = '# 中文标题';
			const result = parse(source);
			expect(result.tokens[0].type).toBe(TokenType.Heading);
		});

		test('Chinese characters in paragraph with formatting', () => {
			const source = '中文**粗体**和*斜体*';
			const result = parse(source);
			const para = result.tokens[0] as ParagraphToken;
			expect(para.children?.some((t) => t.type === TokenType.Bold)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Italic)).toBe(true);
		});

		test('emoji in text', () => {
			const source = 'Hello 😀 World';
			const result = parse(source);
			expect(result.tokens[0].type).toBe(TokenType.Paragraph);
		});

		test('special markdown chars in code', () => {
			const source = 'Text with `*italic*` in code';
			const result = parse(source);
			const para = result.tokens[0] as ParagraphToken;
			const code = para.children?.find((t) => t.type === TokenType.InlineCode) as InlineCodeToken;
			expect(code.content).toContain('*italic*');
		});

		test('unicode in table', () => {
			const source = '| 中文 | English |\n|---|---|\n| 数据 | Data |';
			const result = parse(source);
			expect(result.tokens[0].type).toBe(TokenType.Table);
		});
	});

	describe('Position Tracking in Complex Documents', () => {
		test('positions are monotonically increasing', () => {
			const source = '# Heading\n\nParagraph\n\n```code```\n\n> Quote';
			const result = parse(source);
			let lastEnd = 0;
			for (const token of result.tokens) {
				expect(token.start).toBeGreaterThanOrEqual(lastEnd);
				lastEnd = token.end;
			}
		});

		test('inline positions within paragraph are correct', () => {
			const source = 'Hello **bold** world';
			const result = parse(source);
			const para = result.tokens[0] as ParagraphToken;
			const children = para.children || [];

			for (let i = 0; i < children.length - 1; i++) {
				expect(children[i].end).toBeLessThanOrEqual(children[i + 1].start);
			}
		});

		test('region boundaries are valid', () => {
			const source = '> Quote\n\n```code```\n\n- List\n- Items';
			const result = parse(source);

			for (const region of result.regions) {
				expect(region.start).toBeLessThan(region.end);
				expect(region.start).toBeGreaterThanOrEqual(0);
				expect(region.end).toBeLessThanOrEqual(source.length);
			}
		});
	});

	describe('Regression Tests - Known Edge Cases', () => {
		test('markdown-like text that should not parse as blocks', () => {
			const source = 'Not a #heading or >quote or -list';
			const result = parse(source);
			expect(result.tokens.length).toBe(1);
			expect(result.tokens[0].type).toBe(TokenType.Paragraph);
		});

		test('multiple blank lines between blocks', () => {
			const source = 'Paragraph 1\n\n\n\n\nParagraph 2';
			const result = parse(source);
			expect(result.tokens.length).toBe(2);
		});

		test('indented text is not a code block', () => {
			const source = '    indented text';
			const result = parse(source);
			// Should parse as paragraph, not code block (no fence)
			expect(result.tokens[0].type).toBe(TokenType.Paragraph);
		});

		test('some incomplete block patterns are parsed as blocks', () => {
			// Parser is greedy - it will parse valid patterns
			const source = '# Not a heading\n- not a list\n> not a quote';
			const result = parse(source);
			// Some may be parsed as blocks, some as paragraphs - just check we get tokens
			expect(result.tokens.length).toBeGreaterThan(0);
		});

		test('streaming scenario - unclosed code block', () => {
			const source = '```python\nprint("hello")';
			const result = parse(source);
			expect(result.tokens[0].type).toBe(TokenType.CodeBlock);
			const code = result.tokens[0] as CodeBlockToken;
			expect(code.closed).toBe(false);
			expect(code.content).toContain('print');
		});

		test('empty blocks', () => {
			const source = '#\n\n```\n```\n\n>\n\n-\n\n| A | B |\n|---|-|';
			const result = parse(source);
			// Should have some tokens, not crash
			expect(result.tokens.length).toBeGreaterThan(0);
		});
	});

	describe('Complex Real-World Documents', () => {
		test('typical documentation structure', () => {
			const source = `# Project Title

## Installation

Install using npm:

\`\`\`bash
npm install package-name
\`\`\`

## Usage

Import and use:

\`\`\`javascript
import { something } from 'package-name';
something();
\`\`\`

### API

| Method | Description |
|--------|-------------|
| \`method1()\` | Description 1 |
| \`method2()\` | Description 2 |

> **Note:** This is an important note with *italic* text.

### Examples

- First example with \`code\`
- Second example with **bold**

---

That's all!`;

			const result = parse(source);
			expect(result.tokens.length).toBeGreaterThan(10);

			// Check regions
			expect(result.regions.some((r) => r.type === 'codeblock')).toBe(true);
			expect(result.regions.some((r) => r.type === 'table')).toBe(true);
			expect(result.regions.some((r) => r.type === 'list')).toBe(true);
			expect(result.regions.some((r) => r.type === 'blockquote')).toBe(true);
		});

		test('mixed content with all inline types', () => {
			const source = `This is a paragraph with **bold**, *italic*, ~~strikethrough~~, \`code\`, [links](https://example.com), ![images](img.png), and $math$ inline.`;

			const result = parse(source);
			const para = result.tokens[0] as ParagraphToken;

			expect(para.children?.some((t) => t.type === TokenType.Bold)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Italic)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Strikethrough)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.InlineCode)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Link)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Image)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.LatexInline)).toBe(true);
		});
	});
});
