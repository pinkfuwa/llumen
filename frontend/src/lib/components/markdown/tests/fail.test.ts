import { describe, it, expect } from 'vitest';
import { parseSync } from '../parser/block';
import { AstNodeType } from '../parser/types';
import type { TableNode, TableRowNode, TableCellNode, TextNode } from '../parser/types';

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

describe('regression: lone $ signs and latex in tables', () => {
	it('preserves cell boundaries when $ signs appear across separate cells', () => {
		const md =
			[
				'| SYMBOLS | INPUT |',
				'| :--- | :--- |',
				'| $ | **a** **b** **c** $ |',
				'| \\( A \\) | **d** $ |'
			].join('\n') + '\n';
		const { nodes } = parseSync(md);
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Table);

		const table = nodes[0] as TableNode;
		// Header + 2 data rows
		expect(table.children).toHaveLength(3);

		// Each row should have exactly 2 cells (SYMBOLS and INPUT)
		for (const row of table.children) {
			const tr = row as TableRowNode;
			expect(tr.children).toHaveLength(2);
		}

		// First data row: first cell is lone "$", second cell contains "**a** **b** **c** $"
		const row1Cells = (table.children[1] as TableRowNode).children;
		const cell0 = flattenText((row1Cells[0] as TableCellNode).children);
		expect(cell0).toBe('$');

		const cell1 = flattenText((row1Cells[1] as TableCellNode).children);
		// "**a** **b** **c** $" — bold a, bold b, bold c, then " $"
		expect(cell1).toBe('a b c $');
	});

	it('does not merge cells across lone $ in table', () => {
		const markdown = '| $ | **a** $ |\n' + '| --- | --- |\n' + '| x  | y |\n';
		const { nodes } = parseSync(markdown);
		expect(nodes).toHaveLength(1);

		const table = nodes[0] as TableNode;
		const headerRow = table.children[0] as TableRowNode;
		// If cells were merged, we'd have 1 cell instead of 2
		expect(headerRow.children).toHaveLength(2);

		const cell0Text = flattenText((headerRow.children[0] as TableCellNode).children);
		expect(cell0Text).toBe('$');

		const cell1Text = flattenText((headerRow.children[1] as TableCellNode).children);
		expect(cell1Text).toContain('$');
	});

	it('still protects pipes inside valid $...$ LaTeX in a table cell', () => {
		const markdown = '| Math |\n' + '| --- |\n' + '| $|x|$ |\n';
		const { nodes } = parseSync(markdown);
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Table);

		const table = nodes[0] as TableNode;
		const dataRow = table.children[1] as TableRowNode;
		// Single cell: pipes inside $...$ were not treated as cell separators
		expect(dataRow.children).toHaveLength(1);

		const cell = dataRow.children[0] as TableCellNode;
		expect(cell.children).toHaveLength(1);
		expect(cell.children![0].type).toBe(AstNodeType.LatexInline);
	});

	it('protects pipes inside \\(...\\) in table cells', () => {
		const md = '| A | B |\n| :--- | :--- |\n| \\( 1 + 2 \\) | \\( A \\to b \\) |\n';
		const { nodes } = parseSync(md);
		expect(nodes).toHaveLength(1);
		const table = nodes[0] as TableNode;
		const row = table.children[1] as TableRowNode;
		expect(row.children).toHaveLength(2);

		for (const cell of row.children) {
			const c = cell as TableCellNode;
			const hasLatex = c.children!.some((n) => n.type === AstNodeType.LatexInline);
			expect(hasLatex).toBe(true);
		}
	});
});
