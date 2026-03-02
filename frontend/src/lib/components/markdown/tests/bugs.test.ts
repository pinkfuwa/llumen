import { describe, it, expect } from 'vitest';
import { parseSync } from '../parser/block';
import { AstNodeType } from '../parser/types';
import type {
	TableNode,
	TableRowNode,
	TableCellNode,
	ParagraphNode,
	CodeBlockNode,
	TextNode,
	LatexInlineNode,
	HeadingNode
} from '../parser/types';

describe('regression: LaTeX inside table cells', () => {
	it('pipes inside \\( \\) are not treated as table separators', () => {
		const src = '| Formula | Result |\n|---|---|\n| \\(|x|\\) | 42 |';
		const { nodes } = parseSync(src);
		expect(nodes).toHaveLength(1);
		const table = nodes[0] as TableNode;
		expect(table.type).toBe(AstNodeType.Table);

		const dataRow = table.children[1] as TableRowNode;
		expect(dataRow.children.length).toBeGreaterThanOrEqual(2);

		// The first cell should contain the LaTeX with pipes
		const firstCell = dataRow.children[0] as TableCellNode;
		const hasLatex = firstCell.children.some((c) => c.type === AstNodeType.LatexInline);
		expect(hasLatex).toBe(true);
	});

	it('pipes inside $ $ are not treated as table separators', () => {
		const src = '| Expr | Val |\n|---|---|\n| $|x|$ | 7 |';
		const { nodes } = parseSync(src);
		const table = nodes[0] as TableNode;
		expect(table.type).toBe(AstNodeType.Table);

		const dataRow = table.children[1];
		expect(dataRow.children.length).toBeGreaterThanOrEqual(2);
	});
});

describe('regression: code block after HR', () => {
	it('correctly parses code block following a horizontal rule', () => {
		const src = '---\n\n```js\nconsole.log("hello");\n```';
		const { nodes } = parseSync(src);
		expect(nodes.length).toBeGreaterThanOrEqual(2);
		expect(nodes[0].type).toBe(AstNodeType.HorizontalRule);
		expect(nodes[1].type).toBe(AstNodeType.CodeBlock);
		expect((nodes[1] as CodeBlockNode).language).toBe('js');
	});
});

describe('regression: unclosed formatting', () => {
	it('treats unmatched ** as literal text', () => {
		const { nodes } = parseSync('This has **unclosed bold');
		const p = nodes[0] as ParagraphNode;
		// Should not crash; content should be preserved as text
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

describe('regression: table separator ambiguity', () => {
	it('does not treat --- followed by table as a single HR', () => {
		const src = '| A | B |\n|---|---|\n| 1 | 2 |';
		const { nodes } = parseSync(src);
		// Should parse as table, not HR
		expect(nodes[0].type).toBe(AstNodeType.Table);
	});
});

describe('regression: consecutive blocks', () => {
	it('heading immediately followed by code block', () => {
		const src = '# Title\n```\ncode\n```';
		const { nodes } = parseSync(src);
		expect(nodes[0].type).toBe(AstNodeType.Heading);
		expect(nodes[1].type).toBe(AstNodeType.CodeBlock);
	});

	it('multiple paragraphs separated by blank lines', () => {
		const src = 'Paragraph 1\n\nParagraph 2\n\nParagraph 3';
		const { nodes } = parseSync(src);
		const paragraphs = nodes.filter((n) => n.type === AstNodeType.Paragraph);
		expect(paragraphs).toHaveLength(3);
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
