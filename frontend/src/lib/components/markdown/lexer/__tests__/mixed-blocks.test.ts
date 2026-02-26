import { describe, test, expect } from 'vitest';
import { parse } from '../parser';
import { TokenType } from '../tokens';
import type {
	HeadingToken,
	ParagraphToken,
	BlockquoteToken,
	OrderedListToken,
	UnorderedListToken,
	CodeBlockToken,
	TableToken,
	ListItemToken,
	TextToken
} from '../tokens';

describe('MarkdownParser - Mixed Block Combinations', () => {
	describe('Blockquote containing heading', () => {
		test('> # Heading inside blockquote', () => {
			const result = parse('> # Hello World');
			expect(result.tokens.length).toBe(1);
			expect(result.tokens[0].type).toBe(TokenType.Blockquote);

			const bq = result.tokens[0] as BlockquoteToken;
			expect(bq.children.length).toBeGreaterThan(0);
			expect(bq.children[0].type).toBe(TokenType.Heading);

			const heading = bq.children[0] as HeadingToken;
			expect(heading.level).toBe(1);
		});

		test('> ## Multiple levels in blockquote', () => {
			const result = parse('> # H1\n> ## H2');
			expect(result.tokens[0].type).toBe(TokenType.Blockquote);

			const bq = result.tokens[0] as BlockquoteToken;
			expect(bq.children[0].type).toBe(TokenType.Heading);
			expect((bq.children[0] as HeadingToken).level).toBe(1);
			expect(bq.children[1].type).toBe(TokenType.Heading);
			expect((bq.children[1] as HeadingToken).level).toBe(2);
		});

		test('> Heading without space after > is still parsed', () => {
			const result = parse('>NoSpace');
			expect(result.tokens[0].type).toBe(TokenType.Blockquote);
		});
	});

	describe('Blockquote containing list', () => {
		test('> - Unordered list inside blockquote', () => {
			const result = parse('> - Item 1\n> - Item 2');
			expect(result.tokens[0].type).toBe(TokenType.Blockquote);

			const bq = result.tokens[0] as BlockquoteToken;
			expect(bq.children[0].type).toBe(TokenType.UnorderedList);
		});

		test('> 1. Ordered list inside blockquote', () => {
			const result = parse('> 1. First\n> 2. Second');
			expect(result.tokens[0].type).toBe(TokenType.Blockquote);

			const bq = result.tokens[0] as BlockquoteToken;
			expect(bq.children[0].type).toBe(TokenType.OrderedList);
		});

		test('> Mixed list styles', () => {
			const result = parse('> - item 1\n> 1. item 2');
			expect(result.tokens[0].type).toBe(TokenType.Blockquote);

			const bq = result.tokens[0] as BlockquoteToken;
			expect(bq.children[0].type).toBe(TokenType.UnorderedList);
			expect(bq.children[1].type).toBe(TokenType.OrderedList);
		});
	});

	describe('Blockquote containing blockquote', () => {
		test('> > Nested blockquote', () => {
			const result = parse('> > Nested quote');
			expect(result.tokens[0].type).toBe(TokenType.Blockquote);

			const bq = result.tokens[0] as BlockquoteToken;
			expect(bq.children[0].type).toBe(TokenType.Blockquote);
		});

		test('> > > Deeply nested', () => {
			const result = parse('> > > Deeply nested');
			expect(result.tokens[0].type).toBe(TokenType.Blockquote);

			const bq = result.tokens[0] as BlockquoteToken;
			const nested = bq.children[0] as BlockquoteToken;
			expect(nested.children[0].type).toBe(TokenType.Blockquote);
		});
	});

	describe('Blockquote containing code', () => {
		test('> ```code``` inside blockquote', () => {
			const result = parse('> ```js\nconst x = 1;\n```');
			expect(result.tokens[0].type).toBe(TokenType.Blockquote);

			const bq = result.tokens[0] as BlockquoteToken;
			expect(bq.children[0].type).toBe(TokenType.CodeBlock);

			const code = bq.children[0] as CodeBlockToken;
			expect(code.language).toBe('js');
		});
	});

	describe('Blockquote containing table', () => {
		test('> | table | inside blockquote', () => {
			const result = parse('> | A | B |\n> |---|---|\n> | 1 | 2 |');
			expect(result.tokens[0].type).toBe(TokenType.Blockquote);

			const bq = result.tokens[0] as BlockquoteToken;
			expect(bq.children[0].type).toBe(TokenType.Table);
		});
	});

	describe('Blockquote containing paragraph', () => {
		test('> Multiple paragraphs in blockquote', () => {
			const result = parse('> First para\n>\n> Second para');
			expect(result.tokens[0].type).toBe(TokenType.Blockquote);

			const bq = result.tokens[0] as BlockquoteToken;
			expect(bq.children.length).toBe(2);
			expect(bq.children[0].type).toBe(TokenType.Paragraph);
			expect(bq.children[1].type).toBe(TokenType.Paragraph);
		});
	});

	describe('List containing blockquote', () => {
		test('- > Blockquote inside list item (known limitation)', () => {
			// Known limitation: current parser treats "- >" as text in list item
			// The > is not recognized as starting a blockquote within a list item
			const result = parse('- > Quote inside list');
			expect(result.tokens[0].type).toBe(TokenType.UnorderedList);

			const list = result.tokens[0] as UnorderedListToken;
			expect(list.children[0].type).toBe(TokenType.ListItem);

			const item = list.children[0] as ListItemToken;
			// Currently parsed as text, not blockquote
			expect(item.children.length).toBeGreaterThan(0);
		});

		test('1. > Blockquote inside ordered list (known limitation)', () => {
			// Known limitation: current parser treats "1. >" as text in list item
			const result = parse('1. > Quote in ordered list');
			expect(result.tokens[0].type).toBe(TokenType.OrderedList);

			const list = result.tokens[0] as OrderedListToken;
			const item = list.children[0] as ListItemToken;
			expect(item.children.length).toBeGreaterThan(0);
		});
	});

	describe('Heading followed by blockquote', () => {
		test('# Heading\n> Blockquote', () => {
			const result = parse('# Title\n\n> Quote');
			expect(result.tokens[0].type).toBe(TokenType.Heading);
			expect(result.tokens[1].type).toBe(TokenType.Blockquote);
		});

		test('# Heading immediately followed by >', () => {
			const result = parse('# Title\n> Quote');
			expect(result.tokens[0].type).toBe(TokenType.Heading);
			expect(result.tokens[1].type).toBe(TokenType.Blockquote);
		});
	});

	describe('Table followed by blockquote', () => {
		test('| table |\n> Blockquote after table', () => {
			const result = parse('| A | B |\n|---|---|\n| 1 | 2 |\n\n> Quote');
			expect(result.tokens[0].type).toBe(TokenType.Table);
			expect(result.tokens[1].type).toBe(TokenType.Blockquote);
		});
	});

	describe('Code block followed by blockquote', () => {
		test('```\ncode\n```\n> Quote', () => {
			const result = parse('```\ncode\n```\n\n> Quote');
			expect(result.tokens[0].type).toBe(TokenType.CodeBlock);
			expect(result.tokens[1].type).toBe(TokenType.Blockquote);
		});
	});

	describe('Complex nesting', () => {
		test('> # Heading\n> - List\n> ```code```', () => {
			const result = parse('> # Title\n> - Item 1\n> - Item 2\n> ```js\ncode\n```');
			expect(result.tokens[0].type).toBe(TokenType.Blockquote);

			const bq = result.tokens[0] as BlockquoteToken;
			expect(bq.children[0].type).toBe(TokenType.Heading);
			expect(bq.children[1].type).toBe(TokenType.UnorderedList);
			expect(bq.children[2].type).toBe(TokenType.CodeBlock);
		});

		test('Multiple block types in sequence', () => {
			const result = parse(`# Heading

Paragraph.

> Blockquote

- List item

\`\`\`
code
\`\`\`

| A | B |
|---|---|
| 1 | 2 |`);

			expect(result.tokens[0].type).toBe(TokenType.Heading);
			expect(result.tokens[1].type).toBe(TokenType.Paragraph);
			expect(result.tokens[2].type).toBe(TokenType.Blockquote);
			expect(result.tokens[3].type).toBe(TokenType.UnorderedList);
			expect(result.tokens[4].type).toBe(TokenType.CodeBlock);
			expect(result.tokens[5].type).toBe(TokenType.Table);
		});
	});

	describe('Edge cases', () => {
		test('Empty blockquote', () => {
			const result = parse('>');
			expect(result.tokens[0].type).toBe(TokenType.Blockquote);
		});

		test('Blockquote with only blank line', () => {
			const result = parse('>\n>');
			expect(result.tokens[0].type).toBe(TokenType.Blockquote);
		});

		test('Blockquote ending without newline', () => {
			const result = parse('> Quote without newline');
			expect(result.tokens[0].type).toBe(TokenType.Blockquote);
		});
	});
});
