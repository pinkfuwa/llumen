import { describe, it, expect } from 'vitest';
import { parseSync } from '../parser/block';
import { AstNodeType } from '../parser/types';
import type {
	ParagraphNode,
	TextNode,
	CodeBlockNode,
	TableNode,
	TableRowNode,
	TableCellNode,
	LatexBlockNode
} from '../parser/types';

describe('regression: unclosed formatting', () => {
	it('treats unmatched ** as literal text', () => {
		const { nodes } = parseSync('This has **unclosed bold');
		const p = nodes[0] as ParagraphNode;
		const textContent = flattenText(p.children);
		expect(textContent).toContain('**unclosed bold');
	});

	it('treats unmatched * as literal text', () => {
		const { nodes } = parseSync('This has *unclosed italic');
		const p = nodes[0] as ParagraphNode;
		const textContent = flattenText(p.children);
		expect(textContent).toContain('*unclosed italic');
	});

	it('treats unmatched ~~ as literal text', () => {
		const { nodes } = parseSync('This has ~~unclosed strike');
		const p = nodes[0] as ParagraphNode;
		const textContent = flattenText(p.children);
		expect(textContent).toContain('~~unclosed strike');
	});

	it('treats unmatched backtick as literal text', () => {
		const { nodes } = parseSync('This has `unclosed code');
		const p = nodes[0] as ParagraphNode;
		const textContent = flattenText(p.children);
		expect(textContent).toContain('`unclosed code');
	});
});

describe('regression: empty constructs', () => {
	it('empty code block', () => {
		const { nodes } = parseSync('```\n```');
		expect(nodes).toHaveLength(1);
		const code = nodes[0] as CodeBlockNode;
		expect(code.type).toBe(AstNodeType.CodeBlock);
		expect(code.content).toBe('');
		expect(code.closed).toBe(true);
	});

	it('empty blockquote', () => {
		const { nodes } = parseSync('>');
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Blockquote);
	});

	it('empty list item', () => {
		const { nodes } = parseSync('- ');
		expect(nodes).toHaveLength(1);
	});
});

describe('regression: paragraph then table with single newline', () => {
	it('parses paragraph followed by table on next line', () => {
		const markdown =
			'A line with only single newline followed\n| Notation | Relation Type | Property Example |\n| :--- | :--- | :--- |\n| \\( \\Theta \\) | Equivalence Relation | Has **Symmetry**, Reflexivity, Transitivity |';
		const { nodes } = parseSync(markdown);
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
	it('parses escaped brackets', () => {
		const markdown = 'prefix：\\[ X \\rightarrow Y a \\]';
		const { nodes } = parseSync(markdown);
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Paragraph);

		const textContent = flattenText((nodes[0] as ParagraphNode).children);
		expect(textContent).toContain('prefix：');
		const latexNode = nodes[0].children!.at(-1) as LatexBlockNode;
		expect(latexNode.type).toBe(AstNodeType.LatexBlock);
		expect(latexNode.content).toBe('X \\rightarrow Y a');
	});
});

describe('regression: empty table cells', () => {
	it('handles leading empty cell (consecutive pipes)', () => {
		const markdown = '|a|b|\n|---|----|\n||x|\n';
		const { nodes } = parseSync(markdown);
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Table);

		const table = nodes[0] as TableNode;
		const dataRow = table.children[1] as TableRowNode;
		expect(dataRow.children).toHaveLength(2);
		const cell0 = dataRow.children[0] as TableCellNode;
		expect(cell0.children).toHaveLength(0); // empty first cell
		const cell1 = dataRow.children[1] as TableCellNode;
		const cell1Text = flattenText(cell1.children);
		expect(cell1Text).toBe('x');
	});

	it('handles middle empty cell', () => {
		const markdown = '|a|b|c|\n|---|---|---|\n| a || c |\n';
		const { nodes } = parseSync(markdown);
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Table);

		const table = nodes[0] as TableNode;
		const dataRow = table.children[1] as TableRowNode;
		expect(dataRow.children).toHaveLength(3);
		const cell0 = dataRow.children[0] as TableCellNode;
		expect(flattenText(cell0.children)).toBe('a');
		const cell1 = dataRow.children[1] as TableCellNode;
		expect(cell1.children).toHaveLength(0); // empty middle cell
		const cell2 = dataRow.children[2] as TableCellNode;
		expect(flattenText(cell2.children)).toBe('c');
	});
});

describe('regression: lone dollar sign in table cell', () => {
	it('does not merge table cells when a lone $ appears in a cell', () => {
		const markdown =
			'| A | B | C | D |\n' +
			'| --- | --- | --- | --- |\n' +
			'| 1  | 0 | id + id * id $ | action |\n';
		const { nodes } = parseSync(markdown);
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Table);

		const table = nodes[0] as TableNode;
		expect(table.children).toHaveLength(2); // header + 1 data row

		const dataRow = table.children[1] as TableRowNode;
		expect(dataRow.children).toHaveLength(4);

		// Verify the $ stayed in column C and didn't eat column D
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

	it('protects pipes inside valid LaTeX $...$ in a table cell', () => {
		const markdown = '| Math |\n' + '| --- |\n' + '| $|x|$ |\n';
		const { nodes } = parseSync(markdown);
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Table);

		const table = nodes[0] as TableNode;
		const dataRow = table.children[1] as TableRowNode;
		// Single cell: pipes inside $...$ were not treated as cell separators
		expect(dataRow.children).toHaveLength(1);

		const cell = dataRow.children[0] as TableCellNode;
		expect(cell.type).toBe(AstNodeType.TableCell);
		// Cell contains a LaTeXInline node (not split by the inner |)
		expect(cell.children).toHaveLength(1);
		expect(cell.children![0].type).toBe(AstNodeType.LatexInline);
	});
});

function flattenText(nodes: import('../parser/types').AstNode[]): string {
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
