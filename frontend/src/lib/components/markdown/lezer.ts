import type { Tree, SyntaxNode, TreeFragment } from '@lezer/common';

import { parser as baseParser, GFM } from '@lezer/markdown';
const parser = baseParser.configure(GFM);

/**
 * Parse markdown source to a Lezer syntax tree.
 */
export function parseMarkdown(source: string): Tree {
	return parser.parse(source);
}

export function parseInline(source: string): Tree {
	return parser.parse(source);
}

/**
 * Incrementally parse markdown if newSource is an append of prevSource.
 * Falls back to full parse if not.
 */
export function parseMarkdownIncremental(
	prevTree: Tree,
	prevSource: string,
	newSource: string
): Tree {
	if (!newSource.startsWith(prevSource)) {
		return parser.parse(newSource);
	}

	const addedLength = newSource.length - prevSource.length;
	if (addedLength === 0) {
		return prevTree;
	}

	const fragments: TreeFragment[] = [
		{
			from: 0,
			to: prevTree.length,
			offset: 0,
			tree: prevTree
		} as TreeFragment
	];

	return parser.parse(newSource, fragments);
}

/**
 * Walk the Lezer tree and produce a nested AST structure.
 * Each node contains its type, text, and children.
 */
export function walkTree(tree: Tree | null, source: string): any | null {
	if (!tree) {
		return null;
	}
	const nodeTypes = new Set<string>();
	function walk(node: SyntaxNode): any {
		nodeTypes.add(node.type.name);
		const children = [];
		for (let child = node.firstChild; child; child = child.nextSibling) {
			children.push(walk(child));
		}
		return {
			type: node.type.name,
			from: node.from,
			to: node.to,
			text: source.slice(node.from, node.to),
			children
		};
	}
	const result = walk(tree.topNode);

	return result;
}
