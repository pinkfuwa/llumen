import type { Tree, SyntaxNode, TreeFragment } from '@lezer/common';

import {
	parser as baseParser,
	GFM,
	type InlineContext,
	type MarkdownConfig,
	type DelimiterType
} from '@lezer/markdown';
import { tags } from '@lezer/highlight';

// Mathematical expression node names
const INLINE_MATH_DOLLAR = 'InlineMathDollar';
const INLINE_MATH_BRACKET = 'InlineMathBracket';
const BLOCK_MATH_DOLLAR = 'BlockMathDollar';
const BLOCK_MATH_BRACKET = 'BlockMathBracket';

/**
 * Length of the delimiter for each math expression type
 */
const DELIMITER_LENGTH: Record<string, number> = {
	[INLINE_MATH_DOLLAR]: 1, // $
	[INLINE_MATH_BRACKET]: 2, // \(
	[BLOCK_MATH_DOLLAR]: 2, // $$
	[BLOCK_MATH_BRACKET]: 2 // \[
};

/**
 * Delimiters for math expressions
 */
const DELIMITERS = Object.keys(DELIMITER_LENGTH).reduce<Record<string, DelimiterType>>(
	(agg, name) => {
		agg[name] = { mark: `${name}Mark`, resolve: name };
		return agg;
	},
	{}
);

/**
 * LaTeX math extension for Lezer Markdown.
 * Supports inline math ($...$, \(...\)) and display math ($$...$$, \[...\]).
 */
const latexExtension: MarkdownConfig = {
	defineNodes: [
		// Define node types for each math expression
		{ name: BLOCK_MATH_DOLLAR, style: tags.emphasis },
		{ name: `${BLOCK_MATH_DOLLAR}Mark`, style: tags.processingInstruction },
		{ name: INLINE_MATH_DOLLAR, style: tags.emphasis },
		{ name: `${INLINE_MATH_DOLLAR}Mark`, style: tags.processingInstruction },
		{ name: INLINE_MATH_BRACKET, style: tags.emphasis },
		{ name: `${INLINE_MATH_BRACKET}Mark`, style: tags.processingInstruction },
		{ name: BLOCK_MATH_BRACKET, style: tags.emphasis },
		{ name: `${BLOCK_MATH_BRACKET}Mark`, style: tags.processingInstruction }
	],
	parseInline: [
		// Block math with $$ ... $$
		{
			name: BLOCK_MATH_DOLLAR,
			parse(cx: InlineContext, next: number, pos: number): number {
				if (next !== 36 /* '$' */ || cx.char(pos + 1) !== 36) {
					return -1;
				}

				return cx.addDelimiter(
					DELIMITERS[BLOCK_MATH_DOLLAR],
					pos,
					pos + DELIMITER_LENGTH[BLOCK_MATH_DOLLAR],
					true,
					true
				);
			}
		},
		// Inline math with $ ... $ (must not be followed by another $)
		{
			name: INLINE_MATH_DOLLAR,
			parse(cx: InlineContext, next: number, pos: number): number {
				if (next !== 36 /* '$' */ || cx.char(pos + 1) === 36) {
					return -1;
				}

				// Check for space before $ (except at start)
				if (pos > 0) {
					const prevChar = cx.char(pos - 1);
					if (prevChar !== 32 && prevChar !== 10 && prevChar !== 9) {
						return -1;
					}
				}

				return cx.addDelimiter(
					DELIMITERS[INLINE_MATH_DOLLAR],
					pos,
					pos + DELIMITER_LENGTH[INLINE_MATH_DOLLAR],
					true,
					true
				);
			}
		},
		// Inline math with \( ... \)
		{
			name: INLINE_MATH_BRACKET,
			before: 'Escape',
			parse(cx: InlineContext, next: number, pos: number): number {
				if (next !== 92 /* '\' */ || ![40 /* '(' */, 41 /* ')' */].includes(cx.char(pos + 1))) {
					return -1;
				}

				return cx.addDelimiter(
					DELIMITERS[INLINE_MATH_BRACKET],
					pos,
					pos + DELIMITER_LENGTH[INLINE_MATH_BRACKET],
					cx.char(pos + 1) === 40,
					cx.char(pos + 1) === 41
				);
			}
		},
		// Block math with \[ ... \]
		{
			name: BLOCK_MATH_BRACKET,
			before: 'Escape',
			parse(cx: InlineContext, next: number, pos: number): number {
				if (next !== 92 /* '\' */ || ![91 /* '[' */, 93 /* ']' */].includes(cx.char(pos + 1))) {
					return -1;
				}

				return cx.addDelimiter(
					DELIMITERS[BLOCK_MATH_BRACKET],
					pos,
					pos + DELIMITER_LENGTH[BLOCK_MATH_BRACKET],
					cx.char(pos + 1) === 91,
					cx.char(pos + 1) === 93
				);
			}
		}
	]
};

const parser = baseParser.configure([GFM, latexExtension]);

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
 * Represents a region in the source that cannot be incrementally parsed.
 */
interface BlockRegion {
	start: number;
	end: number;
	type: string;
}

/**
 * A block detector identifies regions in the source that need full re-parsing.
 */
interface BlockDetector {
	name: string;
	detect: (source: string, changeStart: number) => BlockRegion[];
}

/**
 * Detects table blocks in the source.
 * Tables are identified by lines containing pipes and table header separators.
 */
const tableDetector: BlockDetector = {
	name: 'table',
	detect(source: string, changeStart: number): BlockRegion[] {
		const regions: BlockRegion[] = [];
		const lines = source.split('\n');
		let lineStart = 0;
		let inTable = false;
		let tableStart = 0;

		for (let i = 0; i < lines.length; i++) {
			const line = lines[i];
			const lineEnd = lineStart + line.length;

			// Check if line contains table syntax
			const hasTableSyntax = line.includes('|');
			const isHeaderSeparator = /^\s*\|?[\s\-:|]+\|\s*$/.test(line);

			if (hasTableSyntax || isHeaderSeparator) {
				if (!inTable) {
					// Start of a new table
					inTable = true;
					tableStart = lineStart;
				}
			} else if (inTable && line.trim() === '') {
				// Empty line might end the table, but continue checking
				continue;
			} else if (inTable) {
				// Non-table line ends the table
				const tableEnd = lineStart;
				// Only include tables that overlap with or are after the change
				if (tableEnd >= changeStart) {
					regions.push({
						start: tableStart,
						end: tableEnd,
						type: 'table'
					});
				}
				inTable = false;
			}

			lineStart = lineEnd + 1; // +1 for newline
		}

		// Handle table at end of document
		if (inTable) {
			regions.push({
				start: tableStart,
				end: source.length,
				type: 'table'
			});
		}

		return regions;
	}
};

/**
 * Detects code fence blocks in the source.
 * Code fences are identified by lines starting with ``` or ~~~.
 */
const codeFenceDetector: BlockDetector = {
	name: 'codefence',
	detect(source: string, changeStart: number): BlockRegion[] {
		const regions: BlockRegion[] = [];
		const lines = source.split('\n');
		let lineStart = 0;
		let inFence = false;
		let fenceStart = 0;
		let fenceDelimiter = '';

		for (let i = 0; i < lines.length; i++) {
			const line = lines[i];
			const lineEnd = lineStart + line.length;

			const backtickMatch = line.match(/^(`{3,})/);
			const tildeMatch = line.match(/^(~{3,})/);
			const match = backtickMatch || tildeMatch;

			if (match) {
				if (!inFence) {
					// Start of a code fence
					inFence = true;
					fenceStart = lineStart;
					fenceDelimiter = match[1][0]; // '`' or '~'
				} else if (match[1][0] === fenceDelimiter && match[1].length >= 3) {
					// End of code fence
					const fenceEnd = lineEnd + 1; // Include newline
					// Only include fences that overlap with or are after the change
					if (fenceEnd >= changeStart) {
						regions.push({
							start: fenceStart,
							end: fenceEnd,
							type: 'codefence'
						});
					}
					inFence = false;
					fenceDelimiter = '';
				}
			}

			lineStart = lineEnd + 1; // +1 for newline
		}

		// Handle unclosed fence at end of document
		if (inFence) {
			regions.push({
				start: fenceStart,
				end: source.length,
				type: 'codefence'
			});
		}

		return regions;
	}
};

/**
 * Detects LaTeX math blocks in the source for incremental parsing.
 * LaTeX blocks are identified by $$ delimiters or \[ \] pairs.
 * Note: Lezer now parses these natively, but we still exclude them from
 * incremental parsing to ensure correctness.
 */
const latexDetector: BlockDetector = {
	name: 'latex',
	detect(source: string, changeStart: number): BlockRegion[] {
		const regions: BlockRegion[] = [];
		let pos = 0;

		// Scan for $$ delimiters (display math)
		while (pos < source.length) {
			const dollarIndex = source.indexOf('$$', pos);
			if (dollarIndex === -1) break;

			const nextDollarIndex = source.indexOf('$$', dollarIndex + 2);
			if (nextDollarIndex === -1) {
				// Unclosed $$, include from $$ to end
				if (dollarIndex >= changeStart) {
					regions.push({
						start: dollarIndex,
						end: source.length,
						type: 'latex'
					});
				}
				break;
			}

			const blockEnd = nextDollarIndex + 2;
			if (blockEnd >= changeStart) {
				regions.push({
					start: dollarIndex,
					end: blockEnd,
					type: 'latex'
				});
			}
			pos = blockEnd;
		}

		// Scan for \[ \] delimiters (LaTeX blocks)
		pos = 0;
		while (pos < source.length) {
			const openIndex = source.indexOf('\\[', pos);
			if (openIndex === -1) break;

			const closeIndex = source.indexOf('\\]', openIndex + 2);
			if (closeIndex === -1) {
				// Unclosed \[, include from \[ to end
				if (openIndex >= changeStart) {
					regions.push({
						start: openIndex,
						end: source.length,
						type: 'latex'
					});
				}
				break;
			}

			const blockEnd = closeIndex + 2;
			if (blockEnd >= changeStart) {
				regions.push({
					start: openIndex,
					end: blockEnd,
					type: 'latex'
				});
			}
			pos = blockEnd;
		}

		// Scan for \( ... \) inline math
		pos = 0;
		while (pos < source.length) {
			const openIndex = source.indexOf('\\(', pos);
			if (openIndex === -1) break;

			const closeIndex = source.indexOf('\\)', openIndex + 2);
			if (closeIndex === -1) {
				if (openIndex >= changeStart) {
					regions.push({
						start: openIndex,
						end: source.length,
						type: 'latex-inline'
					});
				}
				break;
			}

			const blockEnd = closeIndex + 2;
			if (blockEnd >= changeStart) {
				regions.push({
					start: openIndex,
					end: blockEnd,
					type: 'latex-inline'
				});
			}
			pos = blockEnd;
		}

		// Scan for $ ... $ inline math (space required before and after)
		pos = 0;
		const dollarInlineRegex = /(\s)\$(.+?)\$(\s)/gs;
		let match;
		while ((match = dollarInlineRegex.exec(source)) !== null) {
			const start = match.index + match[1].length;
			const end = dollarInlineRegex.lastIndex - match[3].length;
			if (end >= changeStart) {
				regions.push({
					start,
					end,
					type: 'latex-inline'
				});
			}
			pos = dollarInlineRegex.lastIndex;
		}

		return regions;
	}
};

/**
 * Registry of all block detectors.
 * Add new detectors here to extend support for additional block types.
 */
const blockDetectors: BlockDetector[] = [tableDetector, codeFenceDetector, latexDetector];

/**
 * Finds all block regions in the source that cannot be incrementally parsed.
 */
function findBlockRegions(source: string, changeStart: number): BlockRegion[] {
	const allRegions: BlockRegion[] = [];

	for (const detector of blockDetectors) {
		const regions = detector.detect(source, changeStart);
		allRegions.push(...regions);
	}

	// Sort regions by start position and merge overlapping regions
	allRegions.sort((a, b) => a.start - b.start);

	const merged: BlockRegion[] = [];
	for (const region of allRegions) {
		if (merged.length === 0) {
			merged.push(region);
		} else {
			const last = merged[merged.length - 1];
			if (region.start <= last.end) {
				// Overlapping or adjacent regions, merge them
				last.end = Math.max(last.end, region.end);
				last.type = `${last.type}+${region.type}`;
			} else {
				merged.push(region);
			}
		}
	}

	return merged;
}

/**
 * Builds tree fragments from the previous tree, excluding problematic block regions.
 */
function buildFragments(
	prevTree: Tree,
	prevSource: string,
	newSource: string,
	blockRegions: BlockRegion[]
): TreeFragment[] {
	const fragments: TreeFragment[] = [];
	let lastEnd = 0;

	for (const region of blockRegions) {
		// Add fragment for the region before this block
		if (region.start > lastEnd && lastEnd < prevSource.length) {
			const fragmentEnd = Math.min(region.start, prevSource.length);
			fragments.push({
				from: lastEnd,
				to: fragmentEnd,
				offset: 0,
				tree: prevTree
			} as TreeFragment);
		}
		// Skip the block region (no fragment created)
		lastEnd = Math.max(lastEnd, region.end);
	}

	// Add fragment for the region after the last block
	if (lastEnd < prevSource.length) {
		fragments.push({
			from: lastEnd,
			to: prevSource.length,
			offset: 0,
			tree: prevTree
		} as TreeFragment);
	}

	return fragments;
}

/**
 * Incrementally parse markdown if newSource is an append of prevSource.
 * Falls back to full parse if not, or if problematic blocks are detected.
 *
 * Problematic blocks (tables, code fences, etc.) are excluded from incremental
 * parsing to ensure correctness, while other regions benefit from fragment reuse.
 */
export function parseMarkdownIncremental(
	prevTree: Tree,
	prevSource: string,
	newSource: string
): Tree {
	// Not an append, do full parse
	if (!newSource.startsWith(prevSource)) {
		return parser.parse(newSource);
	}

	const addedLength = newSource.length - prevSource.length;

	// No change, return previous tree
	if (addedLength === 0) {
		return prevTree;
	}

	const changeStart = prevSource.length;

	// Find all problematic block regions that need re-parsing
	const blockRegions = findBlockRegions(newSource, changeStart);

	// If no problematic blocks found, use simple incremental parsing
	if (blockRegions.length === 0) {
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

	// Build fragments excluding problematic block regions
	const fragments = buildFragments(prevTree, prevSource, newSource, blockRegions);

	// If no fragments can be reused, fall back to full parse
	if (fragments.length === 0) {
		return parser.parse(newSource);
	}

	// Parse incrementally with selective fragment reuse
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
	// console.log('Node types found:', Array.from(nodeTypes).sort());
	return result;
}
