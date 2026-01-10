import { describe, test, expect } from 'vitest';
import { parse } from '../parser';
import { TokenType } from '../tokens';
import type {
	HeadingToken,
	CodeBlockToken,
	BlockquoteToken,
	OrderedListToken,
	UnorderedListToken,
	ListItemToken,
	TableToken,
	TableRowToken,
	HorizontalRuleToken,
	ParagraphToken
} from '../tokens';

describe('MarkdownParser - Block-Level Tokens', () => {
	describe('Headings', () => {
		test('parses h1 heading', () => {
			const markdown = '# Heading 1';
			const result = parse(markdown);
			expect(result.tokens.length).toBe(1);
			const heading = result.tokens[0] as HeadingToken;
			expect(heading.type).toBe(TokenType.Heading);
			expect(heading.level).toBe(1);
		});

		test('parses all heading levels', () => {
			const markdown = `# H1
## H2
### H3
#### H4
##### H5
###### H6`;
			const result = parse(markdown);
			expect(result.tokens.length).toBe(6);
			for (let i = 0; i < 6; i++) {
				const heading = result.tokens[i] as HeadingToken;
				expect(heading.type).toBe(TokenType.Heading);
				expect(heading.level).toBe(i + 1);
			}
		});

		test('parses heading with inline formatting', () => {
			const markdown = '# Heading with **bold** and *italic*';
			const result = parse(markdown);
			const heading = result.tokens[0] as HeadingToken;
			expect(heading.children.length).toBeGreaterThan(1);
		});

		test('does not parse heading without space after hash', () => {
			const markdown = '#NoSpace';
			const result = parse(markdown);
			expect(result.tokens[0].type).toBe(TokenType.Paragraph);
		});
	});

	describe('Code Blocks', () => {
		test('parses code block with language', () => {
			const markdown = '```javascript\nconst x = 42;\n```';
			const result = parse(markdown);
			expect(result.tokens.length).toBe(1);
			const codeBlock = result.tokens[0] as CodeBlockToken;
			expect(codeBlock.type).toBe(TokenType.CodeBlock);
			expect(codeBlock.language).toBe('javascript');
			expect(codeBlock.content).toBe('const x = 42;');
		});

		test('parses code block without language', () => {
			const markdown = '```\nplain code\n```';
			const result = parse(markdown);
			const codeBlock = result.tokens[0] as CodeBlockToken;
			expect(codeBlock.type).toBe(TokenType.CodeBlock);
			expect(codeBlock.language).toBeUndefined();
			expect(codeBlock.content).toBe('plain code');
		});

		test('parses code block with multiple lines', () => {
			const markdown = '```python\ndef hello():\n    print("world")\n    return True\n```';
			const result = parse(markdown);
			const codeBlock = result.tokens[0] as CodeBlockToken;
			expect(codeBlock.content).toContain('def hello():');
			expect(codeBlock.content).toContain('print("world")');
		});

		test('preserves whitespace in code blocks', () => {
			const markdown = '```\n  indented\n    more indented\n```';
			const result = parse(markdown);
			const codeBlock = result.tokens[0] as CodeBlockToken;
			expect(codeBlock.content).toBe('  indented\n    more indented');
		});

		test('marks closed code block as closed', () => {
			const markdown = '```javascript\nconst x = 42;\n```';
			const result = parse(markdown);
			const codeBlock = result.tokens[0] as CodeBlockToken;
			expect(codeBlock.closed).toBe(true);
		});

		test('marks unclosed code block as not closed', () => {
			const markdown = '```javascript\nconst x = 42;';
			const result = parse(markdown);
			const codeBlock = result.tokens[0] as CodeBlockToken;
			expect(codeBlock.closed).toBe(false);
			expect(codeBlock.content).toBe('const x = 42;');
		});

		test('marks streaming code block as not closed', () => {
			const markdown = '```python\ndef hello():\n    print("world")';
			const result = parse(markdown);
			const codeBlock = result.tokens[0] as CodeBlockToken;
			expect(codeBlock.closed).toBe(false);
			expect(codeBlock.language).toBe('python');
		});
	});

	describe('Blockquotes', () => {
		test('parses simple blockquote', () => {
			const markdown = '> This is a quote';
			const result = parse(markdown);
			expect(result.tokens.length).toBe(1);
			const blockquote = result.tokens[0] as BlockquoteToken;
			expect(blockquote.type).toBe(TokenType.Blockquote);
			expect(blockquote.children.length).toBeGreaterThan(0);
		});

		test('parses multiline blockquote', () => {
			const markdown = '> Line 1\n> Line 2\n> Line 3';
			const result = parse(markdown);
			const blockquote = result.tokens[0] as BlockquoteToken;
			expect(blockquote.type).toBe(TokenType.Blockquote);
		});

		test('parses blockquote with nested formatting', () => {
			const markdown = '> This has **bold** text';
			const result = parse(markdown);
			const blockquote = result.tokens[0] as BlockquoteToken;
			expect(blockquote.children.length).toBeGreaterThan(0);
		});

		test('parses blockquote with blank line inside', () => {
			const markdown = '> First part\n>\n> Second part';
			const result = parse(markdown);
			expect(result.tokens.length).toBe(1);
			expect(result.tokens[0].type).toBe(TokenType.Blockquote);
		});
	});

	describe('Lists', () => {
		test('parses ordered list', () => {
			const markdown = '1. First\n2. Second\n3. Third';
			const result = parse(markdown);
			expect(result.tokens.length).toBe(1);
			const list = result.tokens[0] as OrderedListToken;
			expect(list.type).toBe(TokenType.OrderedList);
			expect(list.children.length).toBe(3);
		});

		test('parses unordered list with asterisks', () => {
			const markdown = '* Item 1\n* Item 2\n* Item 3';
			const result = parse(markdown);
			const list = result.tokens[0] as UnorderedListToken;
			expect(list.type).toBe(TokenType.UnorderedList);
			expect(list.children.length).toBe(3);
		});

		test('parses unordered list with dashes', () => {
			const markdown = '- Item 1\n- Item 2\n- Item 3';
			const result = parse(markdown);
			const list = result.tokens[0] as UnorderedListToken;
			expect(list.type).toBe(TokenType.UnorderedList);
			expect(list.children.length).toBe(3);
		});

		test('parses list with inline formatting', () => {
			const markdown = '- Item with **bold**\n- Item with *italic*';
			const result = parse(markdown);
			const list = result.tokens[0] as UnorderedListToken;
			expect(list.children.length).toBe(2);
		});

		test('parses ordered list with custom start number', () => {
			const markdown = '5. Fifth\n6. Sixth\n7. Seventh';
			const result = parse(markdown);
			const list = result.tokens[0] as OrderedListToken;
			expect(list.startNumber).toBe(5);
		});

		test('stops list on blank line', () => {
			const markdown = '- Item 1\n- Item 2\n\n- Item 3';
			const result = parse(markdown);
			expect(result.tokens.length).toBe(2);
			expect((result.tokens[0] as UnorderedListToken).children.length).toBe(2);
			expect((result.tokens[1] as UnorderedListToken).children.length).toBe(1);
		});
	});

	describe('Tables', () => {
		test('parses simple table with pipes', () => {
			const markdown = '| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |';
			const result = parse(markdown);
			expect(result.tokens.length).toBe(1);
			const table = result.tokens[0] as TableToken;
			expect(table.type).toBe(TokenType.Table);
			expect(table.children.length).toBe(2); // Header + 1 data row
		});

		test('parses table header row', () => {
			const markdown = '| A | B |\n|---|---|\n| 1 | 2 |';
			const result = parse(markdown);
			const table = result.tokens[0] as TableToken;
			const headerRow = table.children[0] as TableRowToken;
			expect(headerRow.isHeader).toBe(true);
		});

		test('parses table data rows', () => {
			const markdown = '| A | B |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |';
			const result = parse(markdown);
			const table = result.tokens[0] as TableToken;
			expect(table.children.length).toBe(3); // 1 header + 2 data rows
			expect(table.children[1].isHeader).toBe(false);
			expect(table.children[2].isHeader).toBe(false);
		});

		test('parses table with tabs', () => {
			const markdown = 'A\tB\n---\t---\n1\t2';
			const result = parse(markdown);
			const table = result.tokens[0] as TableToken;
			expect(table.type).toBe(TokenType.Table);
			expect(table.children.length).toBe(2);
		});

		test('handles table with inline formatting', () => {
			const markdown = '| **Bold** | *Italic* |\n|----------|----------|\n| Cell     | Cell     |';
			const result = parse(markdown);
			const table = result.tokens[0] as TableToken;
			expect(table.children[0].children.length).toBe(2);
		});

		test('stops table on blank line', () => {
			const markdown = '| A | B |\n|---|---|\n| 1 | 2 |\n\nNot table';
			const result = parse(markdown);
			expect(result.tokens.length).toBe(2);
			expect(result.tokens[0].type).toBe(TokenType.Table);
			expect(result.tokens[1].type).toBe(TokenType.Paragraph);
		});
	});

	describe('Horizontal Rules', () => {
		test('parses horizontal rule with dashes', () => {
			const markdown = '---';
			const result = parse(markdown);
			expect(result.tokens.length).toBe(1);
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

		test('parses long horizontal rule', () => {
			const markdown = '----------';
			const result = parse(markdown);
			expect(result.tokens[0].type).toBe(TokenType.HorizontalRule);
		});

		test('separates content with horizontal rule', () => {
			const markdown = 'Before\n\n---\n\nAfter';
			const result = parse(markdown);
			expect(result.tokens.length).toBe(3);
			expect(result.tokens[0].type).toBe(TokenType.Paragraph);
			expect(result.tokens[1].type).toBe(TokenType.HorizontalRule);
			expect(result.tokens[2].type).toBe(TokenType.Paragraph);
		});
	});

	describe('Mixed block-level tokens', () => {
		test('parses heading followed by paragraph', () => {
			const markdown = '# Title\n\nParagraph text.';
			const result = parse(markdown);
			expect(result.tokens.length).toBe(2);
			expect(result.tokens[0].type).toBe(TokenType.Heading);
			expect(result.tokens[1].type).toBe(TokenType.Paragraph);
		});

		test('parses multiple block types in sequence', () => {
			const markdown = `# Heading

Paragraph

> Quote

- List item

---`;
			const result = parse(markdown);
			expect(result.tokens.length).toBe(5);
			expect(result.tokens[0].type).toBe(TokenType.Heading);
			expect(result.tokens[1].type).toBe(TokenType.Paragraph);
			expect(result.tokens[2].type).toBe(TokenType.Blockquote);
			expect(result.tokens[3].type).toBe(TokenType.UnorderedList);
			expect(result.tokens[4].type).toBe(TokenType.HorizontalRule);
		});

		test('parses code block between paragraphs', () => {
			const markdown = 'Before\n\n```\ncode\n```\n\nAfter';
			const result = parse(markdown);
			expect(result.tokens.length).toBe(3);
			expect(result.tokens[0].type).toBe(TokenType.Paragraph);
			expect(result.tokens[1].type).toBe(TokenType.CodeBlock);
			expect(result.tokens[2].type).toBe(TokenType.Paragraph);
		});
	});
});
