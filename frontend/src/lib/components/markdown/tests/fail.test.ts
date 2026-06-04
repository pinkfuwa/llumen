import { describe, it, expect } from 'vitest';
import { parseSync } from '../vendor/renderer';
import { AstNodeType } from '../vendor/types';
import type { TableNode, TableRowNode, TableCellNode, TextNode, AstNode } from '../vendor/types';

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

describe('regression: lone $ signs and latex in tables', () => {
	it('preserves cell boundaries when $ signs appear across separate cells', () => {
		const md =
			[
				'| SYMBOLS | INPUT |',
				'| :--- | :--- |',
				'| $ | **a** **b** **c** $ |',
				'| \\( A \\) | **d** $ |'
			].join('\n') + '\n';
		const nodes = parseSync(md);
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Table);

		const table = nodes[0] as TableNode;
		expect(table.children).toHaveLength(3);

		for (const row of table.children) {
			const tr = row as TableRowNode;
			expect(tr.children).toHaveLength(2);
		}

		const row1Cells = (table.children[1] as TableRowNode).children;
		const cell0 = flattenText((row1Cells[0] as TableCellNode).children);
		expect(cell0).toContain('$');

		const cell1 = flattenText((row1Cells[1] as TableCellNode).children);
		expect(cell1).toContain('$');
	});

	it('does not merge cells across lone $ in table', () => {
		const markdown = '| $ | **a** $ |\n' + '| --- | --- |\n' + '| x  | y |\n';
		const nodes = parseSync(markdown);
		expect(nodes).toHaveLength(1);

		const table = nodes[0] as TableNode;
		const headerRow = table.children[0] as TableRowNode;
		expect(headerRow.children).toHaveLength(2);

		const cell0Text = flattenText((headerRow.children[0] as TableCellNode).children);
		expect(cell0Text).toContain('$');

		const cell1Text = flattenText((headerRow.children[1] as TableCellNode).children);
		expect(cell1Text).toContain('$');
	});

	it('handles inline latex \\(...\\) in table cells', () => {
		const md = '| A | B |\n| :--- | :--- |\n| \\( 1 + 2 \\) | \\( A \\to b \\) |\n';
		const nodes = parseSync(md);
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
