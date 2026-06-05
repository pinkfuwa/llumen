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
	LatexBlockNode,
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

describe('regression: unclosed formatting on newline', () => {
	it('_ between word chars is literal (not emphasis)', () => {
		const nodes = parseSync('aaaa_unclosedEmphasis\nthis should be normal text.');
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Paragraph);
		const children = (nodes[0] as any).children ?? [];
		const hasItalic = children.some((c: any) => c.type === AstNodeType.Italic);
		expect(hasItalic).toBe(false);
	});

	it('unclosed * emphasis closes on newline', () => {
		const nodes = parseSync('aaaa*unclosedEmphasis\nthis should be normal text.');
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Paragraph);
		const children = (nodes[0] as any).children ?? [];
		const secondLineText = children.find(
			(c: any, i: number) =>
				c.type === AstNodeType.Text && (c as any).content.includes('normal text')
		);
		expect(secondLineText).toBeDefined();
	});

	it('unclosed ** bold closes on newline', () => {
		const nodes = parseSync('aaaa**unclosedBold\nthis should be normal text.');
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Paragraph);
		const children = (nodes[0] as any).children ?? [];
		const secondLineText = children.find(
			(c: any, i: number) =>
				c.type === AstNodeType.Text && (c as any).content.includes('normal text')
		);
		expect(secondLineText).toBeDefined();
	});

	it('unclosed ~~ strikethrough closes on newline', () => {
		const nodes = parseSync('aaaa~~unclosedStrike\nthis should be normal text.');
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Paragraph);
		const children = (nodes[0] as any).children ?? [];
		const secondLineText = children.find(
			(c: any, i: number) =>
				c.type === AstNodeType.Text && (c as any).content.includes('normal text')
		);
		expect(secondLineText).toBeDefined();
	});

	it('unclosed ` code closes on newline', () => {
		const nodes = parseSync('aaaa`unclosedCode\nthis should be normal text.');
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Paragraph);
		const children = (nodes[0] as any).children ?? [];
		const secondLineText = children.find(
			(c: any, i: number) =>
				c.type === AstNodeType.Text && (c as any).content.includes('normal text')
		);
		expect(secondLineText).toBeDefined();
	});
});

describe('regression: unclosed formatting in heading', () => {
	it('closes unclosed ** bold at end of heading', () => {
		const nodes = parseSync('# markdown **content\nThis should be normal text.');
		expect(nodes).toHaveLength(2);
		expect(nodes[0].type).toBe(AstNodeType.Heading);
		expect(nodes[1].type).toBe(AstNodeType.Paragraph);
		expect(flattenText((nodes[1] as ParagraphNode).children ?? [])).toBe(
			'This should be normal text.'
		);
	});

	it('closes unclosed ~~ strikethrough at end of heading', () => {
		const nodes = parseSync('## heading ~~content\nThis should be normal text.');
		expect(nodes).toHaveLength(2);
		expect(nodes[0].type).toBe(AstNodeType.Heading);
		expect(nodes[1].type).toBe(AstNodeType.Paragraph);
		expect(flattenText((nodes[1] as ParagraphNode).children ?? [])).toBe(
			'This should be normal text.'
		);
	});

	it('closes unclosed ~~ with escaped angle brackets in heading', () => {
		const md = "# markdown ~~\\<delete\\>title\nThis should be normal text, but it's normal title";
		const nodes = parseSync(md);
		expect(nodes).toHaveLength(2);
		expect(nodes[0].type).toBe(AstNodeType.Heading);
		expect(nodes[1].type).toBe(AstNodeType.Paragraph);
		const children = (nodes[0] as any).children ?? [];
		const strike = children.find((c: any) => c.type === AstNodeType.Strikethrough);
		expect(strike).toBeDefined();
		const strikeText = flattenText(strike.children ?? []);
		expect(strikeText).toContain('<delete>');
		expect(strikeText).not.toContain('normal text');
	});
});

describe('regression: _ flanking edge cases', () => {
	it('_ at start of word still opens emphasis', () => {
		const nodes = parseSync('hello _world_');
		expect(nodes).toHaveLength(1);
		const children = (nodes[0] as ParagraphNode).children!;
		const italic = children.find((c) => c.type === AstNodeType.Italic);
		expect(italic).toBeDefined();
	});

	it('_ preceded by space is emphasis', () => {
		const nodes = parseSync('hello _world_ again');
		expect(nodes).toHaveLength(1);
		const children = (nodes[0] as ParagraphNode).children!;
		const italic = children.find((c) => c.type === AstNodeType.Italic);
		expect(italic).toBeDefined();
	});

	it('two separate _ emphasis regions', () => {
		const nodes = parseSync('_hello_ _world_');
		expect(nodes).toHaveLength(1);
		const children = (nodes[0] as ParagraphNode).children!;
		const italics = children.filter((c) => c.type === AstNodeType.Italic);
		expect(italics).toHaveLength(2);
	});

	it('_ between numbers is literal', () => {
		const nodes = parseSync('value is 100_000');
		expect(nodes).toHaveLength(1);
		const children = (nodes[0] as ParagraphNode).children!;
		const italic = children.find((c) => c.type === AstNodeType.Italic);
		expect(italic).toBeUndefined();
	});

	it('nested _ inside * still works', () => {
		const nodes = parseSync('*foo _bar_ baz*');
		expect(nodes).toHaveLength(1);
		const children = (nodes[0] as ParagraphNode).children!;
		const outerItalic = children.find((c) => c.type === AstNodeType.Italic);
		expect(outerItalic).toBeDefined();
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
	it('treats \\[ \\] as display math block', () => {
		const markdown = 'prefix：\\[ X \\rightarrow Y a \\]';
		const nodes = parseSync(markdown);
		const para = nodes[0] as ParagraphNode;
		const block = para.children?.find((n) => n.type === AstNodeType.LatexBlock) as
			| LatexBlockNode
			| undefined;
		expect(block).toBeDefined();
		expect(block!.content).toContain('rightarrow');
	});

	it('does not close \\[ block on embedded $$', () => {
		const markdown = '\\[ b$$\\a\\]';
		const nodes = parseSync(markdown);
		const para = nodes[0] as ParagraphNode;
		const block = para.children?.find((n) => n.type === AstNodeType.LatexBlock) as
			| LatexBlockNode
			| undefined;
		expect(block).toBeDefined();
		expect(block!.content).toBe(' b$$\\a');
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

describe('latex delimiter matching', () => {
	it('escaped \\$ inside \\(...\\) produces literal $', () => {
		const nodes = parseSync('\\( a = \\$b \\)');
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Paragraph);
		const para = nodes[0] as ParagraphNode;
		const hasLatex = para.children?.some((c) => c.type === AstNodeType.LatexInline);
		expect(hasLatex).toBe(true);
		const latex = para.children!.find((c) => c.type === AstNodeType.LatexInline)!;
		expect((latex as any).content).toBe(' a = \\$b ');
	});

	it('$ inside \\(...\\) treated as literal (does not close)', () => {
		const nodes = parseSync('\\( a = $10 \\)');
		expect(nodes).toHaveLength(1);
		const para = nodes[0] as ParagraphNode;
		const latex = para.children!.find((c) => c.type === AstNodeType.LatexInline)!;
		expect((latex as any).content).toBe(' a = $10 ');
	});

	it('\\) inside $...$ treated as literal (does not close)', () => {
		const nodes = parseSync('$a \\) b$');
		expect(nodes).toHaveLength(1);
		const para = nodes[0] as ParagraphNode;
		const latex = para.children!.find((c) => c.type === AstNodeType.LatexInline)!;
		expect((latex as any).content).toBe('a \\) b');
	});

	it('lone $ in text followed by space remains literal (not latex)', () => {
		const nodes = parseSync('price is $10');
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Paragraph);
		const text = flattenText((nodes[0] as ParagraphNode).children);
		expect(text).toBe('price is $10');
	});

	it('escaped \\$ in $...$ equation produces literal $', () => {
		const nodes = parseSync('$\\$$');
		expect(nodes).toHaveLength(1);
		const para = nodes[0] as ParagraphNode;
		const latex = para.children!.find((c) => c.type === AstNodeType.LatexInline)!;
		expect((latex as any).content).toBe('\\$');
	});

	it('\\\\$ in $...$ equation closes correctly', () => {
		const nodes = parseSync('$\\\\$');
		expect(nodes).toHaveLength(1);
		const para = nodes[0] as ParagraphNode;
		const latex = para.children!.find((c) => c.type === AstNodeType.LatexInline)!;
		expect((latex as any).content).toBe('\\\\');
	});

	it('backtick inside \\(...\\) does not break equation', () => {
		const nodes = parseSync('\\( \\text{`hello`} \\)');
		expect(nodes).toHaveLength(1);
		const para = nodes[0] as ParagraphNode;
		const latex = para.children!.find((c) => c.type === AstNodeType.LatexInline)!;
		expect((latex as any).content).toBe(' \\text{`hello`} ');
	});

	it('tilde inside \\(...\\) does not open strikethrough', () => {
		const nodes = parseSync('\\( a \\sim b \\)');
		expect(nodes).toHaveLength(1);
		const para = nodes[0] as ParagraphNode;
		const latex = para.children!.find((c) => c.type === AstNodeType.LatexInline)!;
		expect((latex as any).content).toBe(' a \\sim b ');
	});

	it('mixed text and inline latex preserves order', () => {
		const nodes = parseSync('a \\( b \\) c \\( d \\)');
		expect(nodes).toHaveLength(1);
		const para = nodes[0] as ParagraphNode;
		const children = para.children!;
		expect(children).toHaveLength(4);
		expect(children[0].type).toBe(AstNodeType.Text);
		expect((children[0] as TextNode).content).toBe('a ');
		expect(children[1].type).toBe(AstNodeType.LatexInline);
		expect((children[1] as any).content).toBe(' b ');
		expect(children[2].type).toBe(AstNodeType.Text);
		expect((children[2] as TextNode).content).toBe(' c ');
		expect(children[3].type).toBe(AstNodeType.LatexInline);
		expect((children[3] as any).content).toBe(' d ');
	});
});
