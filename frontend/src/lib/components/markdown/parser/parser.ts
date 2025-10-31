import { parser as baseParser } from '@lezer/markdown';
import { lezerLatex } from './lezerLatex';
import { lezerCitation } from './lezerCitation';
import type { Tree, SyntaxNode } from '@lezer/common';
import { TreeFragment } from '@lezer/common';
import type { WorkerToken } from './types';

// Compose the parser with custom extensions
const parser = baseParser.configure([lezerLatex, lezerCitation]);

// Cache for incremental parsing
let lastSource: string | undefined;
let lastTree: Tree | undefined;

/**
 * Parse markdown with incremental support
 */
export function parseMarkdown(source: string): WorkerToken[] {
	let tree: Tree;

	if (lastTree && lastSource !== undefined && lastSource !== source) {
		// Compute tree fragments for incremental parsing
		const fragments = TreeFragment.applyChanges(
			TreeFragment.addTree(lastTree),
			computeChanges(lastSource, source)
		);
		tree = parser.parse(source, fragments);
	} else {
		// Full parse if no previous tree or same source
		tree = parser.parse(source);
	}

	// Cache for next parse
	lastSource = source;
	lastTree = tree;

	// Extract tokens from Lezer tree
	return extractTokens(tree, source);
}

/**
 * Reset the parser cache
 */
export function resetParser(): void {
	lastSource = undefined;
	lastTree = undefined;
}

/**
 * Compute changes between old and new markdown text
 */
function computeChanges(oldText: string, newText: string) {
	const changes: { fromA: number; toA: number; fromB: number; toB: number }[] = [];

	// Simple diff: find first and last differing positions
	let fromA = 0;
	let fromB = 0;

	// Find first difference
	while (fromA < oldText.length && fromB < newText.length && oldText[fromA] === newText[fromB]) {
		fromA++;
		fromB++;
	}

	// If no differences, return empty changes
	if (fromA === oldText.length && fromB === newText.length) {
		return changes;
	}

	// Find last difference
	let toA = oldText.length;
	let toB = newText.length;

	while (toA > fromA && toB > fromB && oldText[toA - 1] === newText[toB - 1]) {
		toA--;
		toB--;
	}

	changes.push({ fromA, toA, fromB, toB });
	return changes;
}

/**
 * Extract tokens from Lezer tree in hierarchical structure
 */
function extractTokens(tree: Tree, source: string): WorkerToken[] {
	const tokens: WorkerToken[] = [];
	const cursor = tree.cursor();

	// Start from Document root and process its children
	if (cursor.firstChild()) {
		do {
			const token = processNode(cursor.node, source);
			if (token) {
				tokens.push(token);
			}
		} while (cursor.nextSibling());
	}

	return tokens;
}

/**
 * Process a single node and its children recursively
 */
function processNode(node: SyntaxNode, source: string): WorkerToken | null {
	const type = node.type.name;
	const from = node.from;
	const to = node.to;
	const raw = source.slice(from, to);

	// Skip empty nodes
	if (from === to) {
		return null;
	}

	// Custom nodes
	if (type === 'BlockKatex') {
		const bracketBlockRegex = /^(\\\[)\n((?:\\[^]|[^\\])+?)\n\\\](?:\n|$)/;
		const parenthesesBlockRegex = /^(\\\()\n((?:\\[^]|[^\\])+?)\n\\\)(?:\n|$)/;
		const dollarBlockRule = /^(\${1,2})\n((?:\\[^]|[^\\])+?)\n\1(?:\n|$)/;
		const match =
			bracketBlockRegex.exec(raw) || parenthesesBlockRegex.exec(raw) || dollarBlockRule.exec(raw);
		const textContent = match ? match[2].trim() : raw;
		const displayMode = match ? match[1].length === 2 : true;
		return {
			type: 'blockKatex',
			raw,
			text: textContent,
			displayMode
		};
	}

	if (type === 'InlineKatex') {
		const bracketInlineRegex = /^(\\\[)(?!\$)((?:\\.|[^\\\n])*?(?:\\.|[^\\\n\$]))\\\]/;
		const parenthesesInlineRegex = /^(\\\()(?!\$)((?:\\.|[^\\\n])*?(?:\\.|[^\\\n\$]))\\\)/;
		const dollarInlineRule =
			/^(\${1,2})(?!\$)((?:\\.|[^\\\n])*?(?:\\.|[^\\\n\$]))\1(?=[\s?!\.,:？！。，：]|$)/;
		const match =
			bracketInlineRegex.exec(raw) ||
			parenthesesInlineRegex.exec(raw) ||
			dollarInlineRule.exec(raw);
		const textContent = match ? match[2].trim() : raw;
		const displayMode = match ? match[1].length === 2 : false;
		return {
			type: 'inlineKatex',
			raw,
			text: textContent,
			displayMode
		};
	}

	if (type === 'Citation') {
		const citationBlockRegex = /^<citation[^>]*>([\s\S]*?)<\/citation>/;
		const fieldRegex = /<(\w+)>([\s\S]*?)<\/\1>/g;
		const match = citationBlockRegex.exec(raw);
		const content = match ? match[1] : '';
		const fields: Record<string, string> = {};
		let fieldMatch;
		while ((fieldMatch = fieldRegex.exec(content)) !== null) {
			const key = fieldMatch[1].toLowerCase();
			const value = fieldMatch[2].trim();
			if (value) fields[key] = value;
		}
		return {
			type: 'citation',
			raw,
			...fields
		};
	}

	// Handle headings
	if (type.startsWith('ATXHeading')) {
		const depth = parseInt(type.replace('ATXHeading', ''));
		const tokens = processChildren(node, source);
		return {
			type: 'heading',
			raw,
			depth,
			tokens
		};
	}

	// Handle paragraphs
	if (type === 'Paragraph') {
		const tokens = processChildren(node, source);
		return {
			type: 'paragraph',
			raw,
			tokens
		};
	}

	// Handle code blocks
	if (type === 'FencedCode' || type === 'CodeBlock') {
		// Extract language and text from fenced code
		const lines = raw.split('\n');
		let lang = '';
		let text = raw;

		if (type === 'FencedCode' && lines.length > 0) {
			const firstLine = lines[0];
			const match = firstLine.match(/^```(\w+)?/);
			if (match) {
				lang = match[1] || '';
				text = lines.slice(1, -1).join('\n');
			}
		}

		return {
			type: 'code',
			raw,
			lang,
			text
		};
	}

	// Handle blockquotes
	if (type === 'Blockquote') {
		const tokens = processChildren(node, source);
		return {
			type: 'blockquote',
			raw,
			tokens
		};
	}

	// Handle lists
	if (type === 'BulletList' || type === 'OrderedList') {
		const items: WorkerToken[] = [];
		const cursor = node.cursor();
		if (cursor.firstChild()) {
			do {
				if (cursor.type.name === 'ListItem') {
					const itemRaw = source.slice(cursor.from, cursor.to);
					const itemTokens = processChildren(cursor.node, source);
					items.push({
						type: 'listitem',
						raw: itemRaw,
						tokens: itemTokens
					});
				}
			} while (cursor.nextSibling());
		}

		return {
			type: 'list',
			raw,
			ordered: type === 'OrderedList',
			items
		};
	}

	// Handle horizontal rule
	if (type === 'HorizontalRule') {
		return {
			type: 'hr',
			raw
		};
	}

	// Handle emphasis
	if (type === 'Emphasis') {
		const tokens = processChildren(node, source);
		return {
			type: 'em',
			raw,
			tokens
		};
	}

	// Handle strong emphasis
	if (type === 'StrongEmphasis') {
		const tokens = processChildren(node, source);
		return {
			type: 'strong',
			raw,
			tokens
		};
	}

	// Handle links
	if (type === 'Link') {
		const cursor = node.cursor();
		let href = '';
		let title = '';
		const tokens: WorkerToken[] = [];

		if (cursor.firstChild()) {
			do {
				const childType = cursor.type.name;
				if (childType === 'URL') {
					href = source.slice(cursor.from, cursor.to);
				} else if (childType === 'LinkTitle') {
					title = source.slice(cursor.from, cursor.to);
				} else {
					const childToken = processNode(cursor.node, source);
					if (childToken) {
						tokens.push(childToken);
					}
				}
			} while (cursor.nextSibling());
		}

		return {
			type: 'link',
			raw,
			href,
			title,
			tokens
		};
	}

	// Handle images
	if (type === 'Image') {
		const cursor = node.cursor();
		let href = '';
		let title = '';
		let text = '';

		if (cursor.firstChild()) {
			do {
				const childType = cursor.type.name;
				if (childType === 'URL') {
					href = source.slice(cursor.from, cursor.to);
				} else if (childType === 'LinkTitle') {
					title = source.slice(cursor.from, cursor.to);
				} else if (childType === 'LinkLabel') {
					text = source.slice(cursor.from, cursor.to);
				}
			} while (cursor.nextSibling());
		}

		return {
			type: 'image',
			raw,
			href,
			title,
			text
		};
	}

	// Handle inline code
	if (type === 'InlineCode') {
		return {
			type: 'codespan',
			raw,
			text: raw
		};
	}

	// Handle line breaks
	if (type === 'HardBreak') {
		return {
			type: 'br',
			raw
		};
	}

	// Handle text nodes
	if (type === 'Text') {
		return {
			type: 'text',
			raw,
			text: raw
		};
	}

	// For other container nodes, process children
	if (node.firstChild) {
		const tokens = processChildren(node, source);
		if (tokens.length > 0) {
			return {
				type: type.toLowerCase(),
				raw,
				tokens
			};
		}
	}

	// Default fallback
	return {
		type: type.toLowerCase(),
		raw
	};
}

/**
 * Process all children of a node
 */
function processChildren(node: SyntaxNode, source: string): WorkerToken[] {
	const tokens: WorkerToken[] = [];
	const cursor = node.cursor();

	if (cursor.firstChild()) {
		do {
			const token = processNode(cursor.node, source);
			if (token) {
				tokens.push(token);
			}
		} while (cursor.nextSibling());
	}

	return tokens;
}
