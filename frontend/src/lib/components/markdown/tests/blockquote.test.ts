import { describe, it, expect } from 'vitest';
import { parseSync } from '../parser/renderer';
import { AstNodeType } from '../parser/types';
import type { ParagraphNode, AstNode } from '../parser/types';

function flattenText(nodes: AstNode[]): string {
	let result = '';
	for (const node of nodes) {
		if (node.type === AstNodeType.Text) {
			result += (node as any).content;
		}
		if (node.children) {
			result += flattenText(node.children);
		}
	}
	return result;
}

describe('blockquote edge cases', () => {
	it('text after blockquote on single newline is not inside blockquote', () => {
		const nodes = parseSync('a\n> b\nc');
		expect(nodes).toHaveLength(3);
		expect(nodes[0].type).toBe(AstNodeType.Paragraph);
		expect(nodes[1].type).toBe(AstNodeType.Blockquote);
		expect(nodes[2].type).toBe(AstNodeType.Paragraph);
		expect(flattenText((nodes[2] as ParagraphNode).children)).toBe('c');
	});

	it('multi-line blockquote still works', () => {
		const nodes = parseSync('> a\n> b');
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Blockquote);
		expect(flattenText(nodes[0].children!)).toBe('ab');
	});

	it('blank line closes blockquote', () => {
		const nodes = parseSync('> a\n\nb');
		expect(nodes).toHaveLength(2);
		expect(nodes[0].type).toBe(AstNodeType.Blockquote);
		expect(nodes[1].type).toBe(AstNodeType.Paragraph);
		expect(flattenText((nodes[1] as ParagraphNode).children)).toBe('b');
	});

	it('single newline after blockquote without leading text', () => {
		const nodes = parseSync('> a\nb');
		expect(nodes).toHaveLength(2);
		expect(nodes[0].type).toBe(AstNodeType.Blockquote);
		expect(nodes[1].type).toBe(AstNodeType.Paragraph);
		expect(flattenText((nodes[1] as ParagraphNode).children)).toBe('b');
	});

	it('nested blockquote still works', () => {
		const nodes = parseSync('> a\n> > b\n> c');
		expect(nodes).toHaveLength(1);
		expect(nodes[0].type).toBe(AstNodeType.Blockquote);
	});

	it('text after nested blockquote on single newline', () => {
		const nodes = parseSync('> > a\nb');
		expect(nodes).toHaveLength(2);
		expect(nodes[0].type).toBe(AstNodeType.Blockquote);
		expect(nodes[1].type).toBe(AstNodeType.Paragraph);
		expect(flattenText((nodes[1] as ParagraphNode).children)).toBe('b');
	});

	it('list inside blockquote then text on single newline', () => {
		const nodes = parseSync('> - a\n> - b\nc');
		expect(nodes).toHaveLength(2);
		expect(nodes[0].type).toBe(AstNodeType.Blockquote);
		expect(nodes[1].type).toBe(AstNodeType.Paragraph);
		expect(flattenText((nodes[1] as ParagraphNode).children)).toBe('c');
	});// known issue: table inside blockquote does not close on single newline (tables handle > internally)

	it('text after blockquote followed by blockquote on single newline produces two blockquotes', () => {
		const nodes = parseSync('> a\n> b\nc\n> d');
		expect(nodes).toHaveLength(3);
		expect(nodes[0].type).toBe(AstNodeType.Blockquote);
		expect(nodes[1].type).toBe(AstNodeType.Paragraph);
		expect(nodes[2].type).toBe(AstNodeType.Blockquote);
	});

	it('single newline before blockquote still starts blockquote', () => {
		const nodes = parseSync('hello\n> quote');
		expect(nodes).toHaveLength(2);
		expect(nodes[0].type).toBe(AstNodeType.Paragraph);
		expect(nodes[1].type).toBe(AstNodeType.Blockquote);
	});
});
