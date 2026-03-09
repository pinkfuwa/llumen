import { describe, it, expect } from 'vitest';
import { parseSync } from '../parser/block';
import { AstNodeType } from '../parser/types';
import type {
	ParagraphNode,
	TextNode,
	CodeBlockNode,
	TableNode,
	TableRowNode
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
