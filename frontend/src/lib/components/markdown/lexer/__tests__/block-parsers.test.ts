import { describe, test, expect } from 'vitest';
import { TokenType } from '../tokens';
import {
	parseHeading,
	parseCodeBlock,
	parseLatexBlock,
	parseHorizontalRule,
	parseTable,
	parseBlockquote,
	parseList,
	parseParagraph,
	parseBlocks,
	type ParseContext
} from '../parsers/block-parser';
import type {
	HeadingToken,
	CodeBlockToken,
	LatexBlockToken,
	HorizontalRuleToken,
	TableToken,
	BlockquoteToken,
	OrderedListToken,
	UnorderedListToken,
	ParagraphToken
} from '../tokens';

describe('BlockParser - Block Parsers', () => {
	const ctx = (source: string, pos: number): ParseContext => ({ source, position: pos });

	describe('parseHeading', () => {
		test('parses h1 heading', () => {
			const result = parseHeading(ctx('# Hello World', 0));
			expect(result.token).not.toBeNull();
			expect((result.token as HeadingToken).type).toBe(TokenType.Heading);
			expect((result.token as HeadingToken).level).toBe(1);
		});

		test('parses h6 heading', () => {
			const result = parseHeading(ctx('###### H6', 0));
			expect((result.token as HeadingToken).level).toBe(6);
		});

		test('parses heading with inline formatting', () => {
			const result = parseHeading(ctx('# **Bold** and *italic*', 0));
			expect((result.token as HeadingToken).children.length).toBeGreaterThan(1);
		});

		test('returns null for heading without space after hash', () => {
			const result = parseHeading(ctx('#NoSpace', 0));
			expect(result.token).toBeNull();
		});

		test('returns null for 7 hashes', () => {
			const result = parseHeading(ctx('####### Too many', 0));
			expect(result.token).toBeNull();
		});

		test('returns null for empty line', () => {
			const result = parseHeading(ctx('', 0));
			expect(result.token).toBeNull();
		});

		test('returns null when not at heading', () => {
			const result = parseHeading(ctx('Not a heading', 0));
			expect(result.token).toBeNull();
		});

		test('handles offset position', () => {
			const source = 'Prefix\n# Heading';
			const result = parseHeading(ctx(source, 7));
			expect(result.token).not.toBeNull();
			expect(result.token!.start).toBe(7);
		});
	});

	describe('parseCodeBlock', () => {
		test('parses code block with language', () => {
			const source = '```javascript\nconst x = 1;\n```';
			const result = parseCodeBlock(ctx(source, 0));
			expect(result.token).not.toBeNull();
			expect((result.token as CodeBlockToken).language).toBe('javascript');
			expect((result.token as CodeBlockToken).content).toBe('const x = 1;');
			expect((result.token as CodeBlockToken).closed).toBe(true);
		});

		test('parses code block without language', () => {
			const source = '```\ncode\n```';
			const result = parseCodeBlock(ctx(source, 0));
			expect((result.token as CodeBlockToken).language).toBeUndefined();
		});

		test('parses unclosed code block (streaming)', () => {
			const source = '```python\nprint("hello")';
			const result = parseCodeBlock(ctx(source, 0));
			expect(result.token).not.toBeNull();
			expect((result.token as CodeBlockToken).closed).toBe(false);
			expect((result.token as CodeBlockToken).content).toBe('print("hello")');
		});

		test('parses multi-line code block', () => {
			const source = '```python\ndef hello():\n    print("world")\n    return True\n```';
			const result = parseCodeBlock(ctx(source, 0));
			expect((result.token as CodeBlockToken).content).toBe(
				'def hello():\n    print("world")\n    return True'
			);
		});

		test('returns null when not at code block', () => {
			const result = parseCodeBlock(ctx('Not code block', 0));
			expect(result.token).toBeNull();
		});

		test('creates region boundary', () => {
			const source = '```js\ncode\n```';
			const result = parseCodeBlock(ctx(source, 0));
			expect(result.regions.length).toBe(1);
			expect(result.regions[0].type).toBe('codeblock');
		});

		test('parses mermaid language', () => {
			const source = '```mermaid\ngraph TD;\nA-->B;\n```';
			const result = parseCodeBlock(ctx(source, 0));
			expect(result.token).not.toBeNull();
			expect((result.token as CodeBlockToken).language).toBe('mermaid');
			expect((result.token as CodeBlockToken).closed).toBe(true);
		});

		test('parses graph language as mermaid', () => {
			const source = '```graph\nTD;\nA-->B;\n```';
			const result = parseCodeBlock(ctx(source, 0));
			expect(result.token).not.toBeNull();
			expect((result.token as CodeBlockToken).language).toBe('graph');
			expect((result.token as CodeBlockToken).closed).toBe(true);
		});

		test('parses flowchart language as mermaid', () => {
			const source = '```flowchart\nTD;\nA-->B;\n```';
			const result = parseCodeBlock(ctx(source, 0));
			expect(result.token).not.toBeNull();
			expect((result.token as CodeBlockToken).language).toBe('flowchart');
		});

		test('parses sequence language as mermaid', () => {
			const source = '```sequence\nAlice->Bob: Hello\n```';
			const result = parseCodeBlock(ctx(source, 0));
			expect(result.token).not.toBeNull();
			expect((result.token as CodeBlockToken).language).toBe('sequence');
		});

		test('parses class language as mermaid', () => {
			const source = '```class\nClass01 <|-- Avery\n```';
			const result = parseCodeBlock(ctx(source, 0));
			expect(result.token).not.toBeNull();
			expect((result.token as CodeBlockToken).language).toBe('class');
		});

		test('parses state language as mermaid', () => {
			const source = '```state\n[*] --> Active\n```';
			const result = parseCodeBlock(ctx(source, 0));
			expect(result.token).not.toBeNull();
			expect((result.token as CodeBlockToken).language).toBe('state');
		});

		test('parses er language as mermaid', () => {
			const source = '```er\nENTITY ||--|{ ENTITY : relates\n```';
			const result = parseCodeBlock(ctx(source, 0));
			expect(result.token).not.toBeNull();
			expect((result.token as CodeBlockToken).language).toBe('er');
		});

		test('parses gantt language as mermaid', () => {
			const source = '```gantt\ntitle Example\nsection Section\nTask1: 2024-01-01, 1d\n```';
			const result = parseCodeBlock(ctx(source, 0));
			expect(result.token).not.toBeNull();
			expect((result.token as CodeBlockToken).language).toBe('gantt');
		});

		test('parses unclosed mermaid code block (streaming)', () => {
			const source = '```graph\nTD;\nA-->';
			const result = parseCodeBlock(ctx(source, 0));
			expect(result.token).not.toBeNull();
			expect((result.token as CodeBlockToken).language).toBe('graph');
			expect((result.token as CodeBlockToken).closed).toBe(false);
		});
	});

	describe('parseLatexBlock', () => {
		test('parses \\[ \\] block', () => {
			const source = '\\[x^2 + y^2\\]';
			const result = parseLatexBlock(ctx(source, 0));
			expect(result.token).not.toBeNull();
			expect((result.token as LatexBlockToken).content).toBe('x^2 + y^2');
		});

		test('parses $$ block with newline', () => {
			const source = '$$\nx^2\n$$';
			const result = parseLatexBlock(ctx(source, 0));
			expect(result.token).not.toBeNull();
		});

		test('returns null for $$ without newline', () => {
			const result = parseLatexBlock(ctx('$$ inline $$', 0));
			expect(result.token).toBeNull();
		});

		test('returns null when not at latex block', () => {
			const result = parseLatexBlock(ctx('Not latex', 0));
			expect(result.token).toBeNull();
		});
	});

	describe('parseHorizontalRule', () => {
		test('parses ---', () => {
			const result = parseHorizontalRule(ctx('---', 0));
			expect(result.token).not.toBeNull();
			expect((result.token as HorizontalRuleToken).type).toBe(TokenType.HorizontalRule);
		});

		test('parses ***', () => {
			const result = parseHorizontalRule(ctx('***', 0));
			expect(result.token).not.toBeNull();
		});

		test('parses ___', () => {
			const result = parseHorizontalRule(ctx('___', 0));
			expect(result.token).not.toBeNull();
		});

		test('returns null for not horizontal rule', () => {
			const result = parseHorizontalRule(ctx('--', 0));
			expect(result.token).toBeNull();
		});
	});

	describe('parseTable', () => {
		test('parses basic table', () => {
			const source = '| A | B |\n|---|---|\n| 1 | 2 |';
			const result = parseTable(ctx(source, 0));
			expect(result.token).not.toBeNull();
			expect((result.token as TableToken).type).toBe(TokenType.Table);
		});

		test('parses tab-separated table', () => {
			const source = 'A\tB\n---\t---\n1\t2';
			const result = parseTable(ctx(source, 0));
			expect(result.token).not.toBeNull();
		});

		test('returns null for non-table', () => {
			const result = parseTable(ctx('Not a table', 0));
			expect(result.token).toBeNull();
		});

		test('creates region boundary', () => {
			const source = '| A | B |\n|---|---|\n| 1 | 2 |';
			const result = parseTable(ctx(source, 0));
			expect(result.regions.length).toBe(1);
			expect(result.regions[0].type).toBe('table');
		});
	});

	describe('parseBlockquote', () => {
		test('parses single-line blockquote', () => {
			const source = '> Quote text';
			const result = parseBlockquote(ctx(source, 0));
			expect(result.token).not.toBeNull();
			expect((result.token as BlockquoteToken).type).toBe(TokenType.Blockquote);
		});

		test('parses multi-line blockquote', () => {
			const source = '> Line 1\n> Line 2\n> Line 3';
			const result = parseBlockquote(ctx(source, 0));
			expect(result.token).not.toBeNull();
		});

		test('returns null for non-blockquote', () => {
			const result = parseBlockquote(ctx('Not a quote', 0));
			expect(result.token).toBeNull();
		});

		test('creates region boundary', () => {
			const source = '> Quote';
			const result = parseBlockquote(ctx(source, 0));
			expect(result.regions.length).toBe(1);
			expect(result.regions[0].type).toBe('blockquote');
		});
	});

	describe('parseList', () => {
		test('parses unordered list with dash', () => {
			const source = '- Item 1\n- Item 2';
			const result = parseList(ctx(source, 0));
			expect(result.token).not.toBeNull();
			expect((result.token as UnorderedListToken).type).toBe(TokenType.UnorderedList);
		});

		test('parses ordered list', () => {
			const source = '1. First\n2. Second';
			const result = parseList(ctx(source, 0));
			expect(result.token).not.toBeNull();
			expect((result.token as OrderedListToken).type).toBe(TokenType.OrderedList);
		});

		test('parses ordered list starting at non-1', () => {
			const source = '5. Fifth\n6. Sixth';
			const result = parseList(ctx(source, 0));
			expect((result.token as OrderedListToken).startNumber).toBe(5);
		});

		test('returns null for non-list', () => {
			const result = parseList(ctx('Not a list', 0));
			expect(result.token).toBeNull();
		});

		test('creates region boundary', () => {
			const source = '- Item';
			const result = parseList(ctx(source, 0));
			expect(result.regions.length).toBe(1);
			expect(result.regions[0].type).toBe('list');
		});
	});

	describe('parseParagraph', () => {
		test('parses simple paragraph', () => {
			const result = parseParagraph(ctx('Just some text', 0));
			expect(result.token).not.toBeNull();
			expect((result.token as ParagraphToken).type).toBe(TokenType.Paragraph);
		});

		test('parses multi-line paragraph', () => {
			const source = 'Line one\nLine two';
			const result = parseParagraph(ctx(source, 0));
			expect(result.token).not.toBeNull();
		});

		test('stops at blank line', () => {
			const source = 'Line one\n\nLine two';
			const result = parseParagraph(ctx(source, 0));
			expect(result.token).not.toBeNull();
		});

		test('creates region boundary', () => {
			const result = parseParagraph(ctx('Paragraph text', 0));
			expect(result.regions.length).toBe(1);
			expect(result.regions[0].type).toBe('paragraph');
		});
	});

	describe('parseBlocks (integration)', () => {
		test('parses multiple blocks', () => {
			const source = '# Heading\n\nParagraph\n\n```code```';
			const result = parseBlocks(source);
			expect(result.tokens.length).toBe(3);
		});

		test('returns empty for empty source', () => {
			const result = parseBlocks('');
			expect(result.tokens.length).toBe(0);
		});

		test('collects regions from all blocks', () => {
			const source = '> Quote\n\n```js\ncode\n```';
			const result = parseBlocks(source);
			expect(result.regions.length).toBe(2);
		});

		test('handles start position offset', () => {
			const source = 'Prefix\n# Heading';
			const result = parseBlocks(source, 7);
			expect(result.tokens.length).toBe(1);
			expect(result.tokens[0].start).toBe(7);
		});

		test('handles Chinese text', () => {
			const source = '這是中文段落';
			const result = parseBlocks(source);
			expect(result.tokens.length).toBe(1);
		});
	});
});
