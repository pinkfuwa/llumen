import { describe, it, expect } from 'vitest';
import { parseSync } from '../parser/block';
import { AstNodeType } from '../parser/types';
import type { TableNode, TableRowNode, TableCellNode, TextNode } from '../parser/types';
import { readFileSync } from 'fs';
import { resolve } from 'path';

const FAIL = readFileSync(resolve(__dirname, '../../../../../../FAIL.md'), 'utf-8');

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

describe('regression: FAIL.md table with mixed $ and latex', () => {
	it('parses the table correctly', () => {
		const { nodes } = parseSync(FAIL);
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Table);

		const table = nodes[0] as TableNode;
		expect(table.children).toHaveLength(8);

		const header = table.children[0] as TableRowNode;
		expect(header.children).toHaveLength(5);

		for (const row of table.children) {
			const tr = row as TableRowNode;
			expect(tr.children).not.toEqual(null);
		}
	});

	it('does not merge cells across lone $ in table', () => {
		const markdown = '| $ | **a** $ |\n' + '| --- | --- |\n' + '| x  | y |\n';
		const { nodes } = parseSync(markdown);
		expect(nodes).toHaveLength(1);

		const table = nodes[0] as TableNode;
		const headerRow = table.children[0] as TableRowNode;
		expect(headerRow.children).toHaveLength(2);

		const cell0Text = flattenText((headerRow.children[0] as TableCellNode).children);
		expect(cell0Text).toBe('$');

		const cell1Text = flattenText((headerRow.children[1] as TableCellNode).children);
		expect(cell1Text).toContain('$');
	});
});
