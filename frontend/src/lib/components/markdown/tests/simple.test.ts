import { describe, it, expect } from 'vitest';
import { parseSync } from '../parser/block';
import { AstNodeType } from '../parser/types';
import type {
	HeadingNode,
	ParagraphNode,
	CodeBlockNode,
	BlockquoteNode,
	OrderedListNode,
	UnorderedListNode,
	ListItemNode,
	TableNode,
	TableRowNode,
	HorizontalRuleNode,
	LatexBlockNode,
	TextNode,
	BoldNode,
	ItalicNode,
	StrikethroughNode,
	InlineCodeNode,
	LinkNode,
	ImageNode,
	LatexInlineNode,
	LineBreakNode
} from '../parser/types';

describe('parseSync: block-level nodes', () => {
	it('parses a heading', () => {
		const { nodes } = parseSync('## Hello world');
		expect(nodes).toHaveLength(1);
		const h = nodes[0] as HeadingNode;
		expect(h.type).toBe(AstNodeType.Heading);
		expect(h.level).toBe(2);
		expect(h.children).toHaveLength(1);
		expect((h.children[0] as TextNode).content).toBe('Hello world');
	});

	it('parses all heading levels', () => {
		for (let level = 1; level <= 6; level++) {
			const { nodes } = parseSync(`${'#'.repeat(level)} Heading ${level}`);
			expect(nodes).toHaveLength(1);
			const h = nodes[0] as HeadingNode;
			expect(h.level).toBe(level);
		}
	});

	it('parses a paragraph', () => {
		const { nodes } = parseSync('Hello world');
		expect(nodes).toHaveLength(1);
		const p = nodes[0] as ParagraphNode;
		expect(p.type).toBe(AstNodeType.Paragraph);
		expect(p.children).toHaveLength(1);
		expect((p.children[0] as TextNode).content).toBe('Hello world');
	});

	it('parses a fenced code block', () => {
		const { nodes } = parseSync('```js\nconsole.log("hi");\n```');
		expect(nodes).toHaveLength(1);
		const code = nodes[0] as CodeBlockNode;
		expect(code.type).toBe(AstNodeType.CodeBlock);
		expect(code.language).toBe('js');
		expect(code.content).toBe('console.log("hi");');
		expect(code.closed).toBe(true);
	});

	it('handles unclosed code block', () => {
		const { nodes } = parseSync('```python\nprint("hello")');
		expect(nodes).toHaveLength(1);
		const code = nodes[0] as CodeBlockNode;
		expect(code.type).toBe(AstNodeType.CodeBlock);
		expect(code.language).toBe('python');
		expect(code.closed).toBe(false);
	});

	it('parses a horizontal rule', () => {
		const { nodes } = parseSync('---');
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.HorizontalRule);
	});

	it('parses a blockquote', () => {
		const { nodes } = parseSync('> This is quoted');
		expect(nodes).toHaveLength(1);
		const bq = nodes[0] as BlockquoteNode;
		expect(bq.type).toBe(AstNodeType.Blockquote);
		expect(bq.children.length).toBeGreaterThan(0);
	});

	it('parses an unordered list', () => {
		const { nodes } = parseSync('- Item 1\n- Item 2\n- Item 3');
		expect(nodes).toHaveLength(1);
		const list = nodes[0] as UnorderedListNode;
		expect(list.type).toBe(AstNodeType.UnorderedList);
		expect(list.children).toHaveLength(3);
		expect(list.children[0].type).toBe(AstNodeType.ListItem);
	});

	it('parses an ordered list', () => {
		const { nodes } = parseSync('1. First\n2. Second\n3. Third');
		expect(nodes).toHaveLength(1);
		const list = nodes[0] as OrderedListNode;
		expect(list.type).toBe(AstNodeType.OrderedList);
		expect(list.children).toHaveLength(3);
		expect(list.startNumber).toBe(1);
	});

	it('parses a table', () => {
		const src = '| A | B |\n|---|---|\n| 1 | 2 |';
		const { nodes } = parseSync(src);
		expect(nodes).toHaveLength(1);
		const table = nodes[0] as TableNode;
		expect(table.type).toBe(AstNodeType.Table);
		expect(table.children).toHaveLength(2); // header + 1 data row
		expect(table.children[0].isHeader).toBe(true);
		expect(table.children[1].isHeader).toBe(false);
	});

	it('parses LaTeX block with $$', () => {
		const { nodes } = parseSync('$$\nx^2 + y^2 = z^2\n$$');
		expect(nodes).toHaveLength(1);
		const latex = nodes[0] as LatexBlockNode;
		expect(latex.type).toBe(AstNodeType.LatexBlock);
		expect(latex.content).toContain('x^2');
	});

	it('parses LaTeX block with \\[ \\]', () => {
		const { nodes } = parseSync('\\[\nE = mc^2\n\\]');
		expect(nodes).toHaveLength(1);
		const latex = nodes[0] as LatexBlockNode;
		expect(latex.type).toBe(AstNodeType.LatexBlock);
		expect(latex.content).toContain('E = mc^2');
	});
});

describe('parseSync: inline nodes', () => {
	it('parses bold text', () => {
		const { nodes } = parseSync('**bold**');
		const p = nodes[0] as ParagraphNode;
		expect(p.children).toHaveLength(1);
		const bold = p.children[0] as BoldNode;
		expect(bold.type).toBe(AstNodeType.Bold);
		expect((bold.children[0] as TextNode).content).toBe('bold');
	});

	it('parses italic text', () => {
		const { nodes } = parseSync('*italic*');
		const p = nodes[0] as ParagraphNode;
		const italic = p.children[0] as ItalicNode;
		expect(italic.type).toBe(AstNodeType.Italic);
		expect((italic.children[0] as TextNode).content).toBe('italic');
	});

	it('parses strikethrough text', () => {
		const { nodes } = parseSync('~~deleted~~');
		const p = nodes[0] as ParagraphNode;
		const del = p.children[0] as StrikethroughNode;
		expect(del.type).toBe(AstNodeType.Strikethrough);
		expect((del.children[0] as TextNode).content).toBe('deleted');
	});

	it('parses inline code', () => {
		const { nodes } = parseSync('`code`');
		const p = nodes[0] as ParagraphNode;
		const code = p.children[0] as InlineCodeNode;
		expect(code.type).toBe(AstNodeType.InlineCode);
		expect(code.content).toBe('code');
	});

	it('parses a link', () => {
		const { nodes } = parseSync('[text](https://example.com)');
		const p = nodes[0] as ParagraphNode;
		const link = p.children[0] as LinkNode;
		expect(link.type).toBe(AstNodeType.Link);
		expect(link.url).toBe('https://example.com');
	});

	it('parses an autolink', () => {
		const { nodes } = parseSync('<https://example.com>');
		const p = nodes[0] as ParagraphNode;
		const link = p.children[0] as LinkNode;
		expect(link.type).toBe(AstNodeType.Link);
		expect(link.url).toBe('https://example.com');
	});

	it('parses an image', () => {
		const { nodes } = parseSync('![alt text](https://img.png)');
		const p = nodes[0] as ParagraphNode;
		const img = p.children[0] as ImageNode;
		expect(img.type).toBe(AstNodeType.Image);
		expect(img.url).toBe('https://img.png');
		expect(img.alt).toBe('alt text');
	});

	it('parses inline LaTeX with \\( \\)', () => {
		const { nodes } = parseSync('The equation \\(E = mc^2\\) is famous');
		const p = nodes[0] as ParagraphNode;
		const latex = p.children.find((n) => n.type === AstNodeType.LatexInline) as LatexInlineNode;
		expect(latex).toBeDefined();
		expect(latex.content).toBe('E = mc^2');
	});

	it('parses line break <br>', () => {
		const { nodes } = parseSync('line1<br>line2');
		const p = nodes[0] as ParagraphNode;
		const br = p.children.find((n) => n.type === AstNodeType.LineBreak) as LineBreakNode;
		expect(br).toBeDefined();
	});
});

describe('parseSync: mixed content', () => {
	it('parses multiple blocks', () => {
		const src = '# Title\n\nParagraph text\n\n---\n\n- item';
		const { nodes } = parseSync(src);
		expect(nodes.length).toBeGreaterThanOrEqual(4);
		expect(nodes[0].type).toBe(AstNodeType.Heading);
		expect(nodes[1].type).toBe(AstNodeType.Paragraph);
		expect(nodes[2].type).toBe(AstNodeType.HorizontalRule);
		expect(nodes[3].type).toBe(AstNodeType.UnorderedList);
	});

	it('parses nested bold and italic', () => {
		const { nodes } = parseSync('**bold *and italic***');
		const p = nodes[0] as ParagraphNode;
		const bold = p.children[0] as BoldNode;
		expect(bold.type).toBe(AstNodeType.Bold);
		// Inner content should contain italic
		const hasItalic = bold.children.some((c) => c.type === AstNodeType.Italic);
		expect(hasItalic).toBe(true);
	});

	it('returns region boundaries', () => {
		const src = '```js\ncode\n```\n\n| a | b |\n|---|---|\n| 1 | 2 |';
		const { regions } = parseSync(src);
		expect(regions.length).toBeGreaterThanOrEqual(2);
		expect(regions.some((r) => r.type === 'codeblock')).toBe(true);
		expect(regions.some((r) => r.type === 'table')).toBe(true);
	});

	it('handles empty input', () => {
		const { nodes } = parseSync('');
		expect(nodes).toHaveLength(0);
	});

	it('handles whitespace-only input', () => {
		const { nodes } = parseSync('   \n\n   ');
		expect(nodes).toHaveLength(0);
	});
});
