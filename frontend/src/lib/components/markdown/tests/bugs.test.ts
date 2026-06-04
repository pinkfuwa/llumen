import { describe, it, expect } from 'vitest';
import { parseSync } from '../vendor/renderer';
import { AstNodeType } from '../vendor/types';
import type {
	ParagraphNode,
	TextNode,
	CodeBlockNode,
	TableNode,
	TableRowNode,
	TableCellNode,
	AstNode
} from '../vendor/types';

function flattenText(nodes: AstNode[]): string {
	let result = '';
	for (const node of nodes) {
		if (node.type === AstNodeType.Text) {
			result += (node as TextNode).content;
		}
		if (node.children) {
			result += flattenText(node.children);
		}
	}
	return result;
}

describe('regression: unclosed formatting', () => {
	it('treats unmatched ** as optimistic bold (does not crash)', () => {
		const nodes = parseSync('This has **unclosed bold');
		expect(nodes.length).toBeGreaterThan(0);
	});

	it('treats unmatched * as optimistic italic (does not crash)', () => {
		const nodes = parseSync('This has *unclosed italic');
		expect(nodes.length).toBeGreaterThan(0);
	});

	it('treats unmatched ~~ as optimistic strikethrough (does not crash)', () => {
		const nodes = parseSync('This has ~~unclosed strike');
		expect(nodes.length).toBeGreaterThan(0);
	});

	it('treats unmatched backtick as optimistic code (does not crash)', () => {
		const nodes = parseSync('This has `unclosed code');
		expect(nodes.length).toBeGreaterThan(0);
	});
});

describe('regression: empty constructs', () => {
	it('empty code block', () => {
		const nodes = parseSync('```\n```');
		expect(nodes).toHaveLength(1);
		const code = nodes[0] as CodeBlockNode;
		expect(code.type).toBe(AstNodeType.CodeBlock);
		expect(code.closed).toBe(true);
	});

	it('empty blockquote', () => {
		const nodes = parseSync('>');
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Blockquote);
	});

	it('empty list item', () => {
		const nodes = parseSync('- ');
		expect(nodes).toHaveLength(1);
	});
});

describe('regression: paragraph then table with single newline', () => {
	it('parses paragraph followed by table on next line', () => {
		const markdown =
			'A line with only single newline followed\n| Notation | Relation Type | Property Example |\n| :--- | :--- | :--- |\n| \\( \\Theta \\) | Equivalence Relation | Has **Symmetry**, Reflexivity, Transitivity |';
		const nodes = parseSync(markdown);
		expect(nodes).toHaveLength(2);
		expect(nodes[0].type).toBe(AstNodeType.Paragraph);
		expect(nodes[1].type).toBe(AstNodeType.Table);

		const table = nodes[1] as TableNode;
		expect(table.children).toHaveLength(2); // header + 1 row

		const headerRow = table.children[0] as TableRowNode;
		expect(headerRow.children).toHaveLength(3);
	});
});

describe('regression: escaped latex brackets', () => {
	it('treats \\[ \\] as literal text (not latex block)', () => {
		// streaming-markdown uses $$ $$ for block equations, not \[ \]
		const markdown = 'prefix：\\[ X \\rightarrow Y a \\]';
		const nodes = parseSync(markdown);
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Paragraph);

		const textContent = flattenText((nodes[0] as ParagraphNode).children);
		expect(textContent).toContain('prefix：');
		expect(textContent).toContain('rightarrow');
	});

	it('parses $$ block equation', () => {
		const markdown = '$$\nX\n$$\n';
		const nodes = parseSync(markdown);
		const hasLatexBlock = nodes.some(
			(n) =>
				n.type === AstNodeType.LatexBlock ||
				n.children?.some((c) => c.type === AstNodeType.LatexBlock)
		);
		expect(hasLatexBlock).toBe(true);
	});

	it('parses \\( \\) inline latex', () => {
		const markdown = 'prefix：\\( X + Y \\)';
		const nodes = parseSync(markdown);
		const hasLatexInline = (nodes[0] as ParagraphNode).children?.some(
			(n) => n.type === AstNodeType.LatexInline
		);
		expect(hasLatexInline).toBe(true);
	});
});

describe('regression: empty table cells', () => {
	it('handles leading empty cell (consecutive pipes)', () => {
		const markdown = '|a|b|\n|---|----|\n||x|\n';
		const nodes = parseSync(markdown);
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Table);

		const table = nodes[0] as TableNode;
		const dataRow = table.children[1] as TableRowNode;
		expect(dataRow.children).toHaveLength(2);
	});

	it('handles middle empty cell', () => {
		const markdown = '|a|b|c|\n|---|---|---|\n| a || c |\n';
		const nodes = parseSync(markdown);
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Table);

		const table = nodes[0] as TableNode;
		const dataRow = table.children[1] as TableRowNode;
		expect(dataRow.children).toHaveLength(3);
	});
});

describe('regression: table cell boundaries', () => {
	it('does not merge table cells when a lone $ appears in a cell', () => {
		const markdown =
			'| A | B | C | D |\n' +
			'| --- | --- | --- | --- |\n' +
			'| 1  | 0 | id + id * id $ | action |\n';
		const nodes = parseSync(markdown);
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Table);

		const table = nodes[0] as TableNode;
		expect(table.children).toHaveLength(2); // header + 1 data row

		const dataRow = table.children[1] as TableRowNode;
		expect(dataRow.children).toHaveLength(4);

		const cellC = dataRow.children[2] as TableCellNode;
		expect(cellC.type).toBe(AstNodeType.TableCell);
		const cellCText = flattenText(cellC.children);
		expect(cellCText).toContain('$');
		expect(cellCText).not.toContain('action');

		const cellD = dataRow.children[3] as TableCellNode;
		expect(cellD.type).toBe(AstNodeType.TableCell);
		const cellDText = flattenText(cellD.children);
		expect(cellDText).toContain('action');
	});
});

describe('single newline behavior', () => {
	it('single newline produces inline LineBreak inside same paragraph', () => {
		const nodes = parseSync('hello\nworld');
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Paragraph);
		const para = nodes[0] as ParagraphNode;
		expect(para.children).toHaveLength(3);
		expect(para.children[0].type).toBe(AstNodeType.Text);
		expect((para.children[0] as TextNode).content).toBe('hello');
		expect(para.children[1].type).toBe(AstNodeType.LineBreak);
		expect(para.children[2].type).toBe(AstNodeType.Text);
		expect((para.children[2] as TextNode).content).toBe('world');
	});

	it('double newline still produces separate paragraphs', () => {
		const nodes = parseSync('hello\n\nworld');
		expect(nodes).toHaveLength(2);
		expect(nodes[0].type).toBe(AstNodeType.Paragraph);
		expect(nodes[1].type).toBe(AstNodeType.Paragraph);
		const text0 = flattenText((nodes[0] as ParagraphNode).children);
		const text1 = flattenText((nodes[1] as ParagraphNode).children);
		expect(text0).toBe('hello');
		expect(text1).toBe('world');
	});

	it('multiple single newlines produce multiple LineBreaks in one paragraph', () => {
		const nodes = parseSync('first line\nsecond line\nthird line');
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Paragraph);
		const para = nodes[0] as ParagraphNode;
		expect(para.children).toHaveLength(5);
		expect((para.children[0] as TextNode).content).toBe('first line');
		expect(para.children[1].type).toBe(AstNodeType.LineBreak);
		expect((para.children[2] as TextNode).content).toBe('second line');
		expect(para.children[3].type).toBe(AstNodeType.LineBreak);
		expect((para.children[4] as TextNode).content).toBe('third line');
	});

	it('two trailing spaces still produces hard break', () => {
		const nodes = parseSync('hello  \nworld');
		expect(nodes).toHaveLength(1);
		const para = nodes[0] as ParagraphNode;
		const hasLineBreak = para.children?.some((c) => c.type === AstNodeType.LineBreak);
		expect(hasLineBreak).toBe(true);
	});

	it('backslash newline still produces hard break', () => {
		const nodes = parseSync('hello\\\nworld');
		expect(nodes).toHaveLength(1);
		const para = nodes[0] as ParagraphNode;
		const hasLineBreak = para.children?.some((c) => c.type === AstNodeType.LineBreak);
		expect(hasLineBreak).toBe(true);
	});

	it('single newline before blockquote still starts blockquote', () => {
		const nodes = parseSync('hello\n> quote');
		expect(nodes).toHaveLength(2);
		expect(nodes[0].type).toBe(AstNodeType.Paragraph);
		expect(nodes[1].type).toBe(AstNodeType.Blockquote);
	});
});
