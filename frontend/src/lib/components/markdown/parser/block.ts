import { LexTokenKind, type LexToken } from '../lexer/types';
import { lexBlock } from '../lexer/block';
import {
	AstNodeType,
	type AstNode,
	type HeadingNode,
	type ParagraphNode,
	type CodeBlockNode,
	type BlockquoteNode,
	type OrderedListNode,
	type UnorderedListNode,
	type ListItemNode,
	type TableNode,
	type TableRowNode,
	type TableCellNode,
	type HorizontalRuleNode,
	type LatexBlockNode,
	type ParseResult,
	type RegionBoundary
} from './types';
import { buildInlineAst } from './inline';

/**
 * Parse markdown source into an AST.
 * Phase 1: lex source into flat tokens.
 * Phase 2: group lex tokens into block-level AST nodes.
 */
export function parseSync(source: string): ParseResult {
	const lexTokens = lexBlock(source);
	return buildBlockAst(lexTokens, source);
}

/**
 * Build block-level AST from a flat list of lex tokens.
 */
export function buildBlockAst(tokens: LexToken[], source: string): ParseResult {
	const nodes: AstNode[] = [];
	const regions: RegionBoundary[] = [];
	let i = 0;

	while (i < tokens.length) {
		// Skip blank lines and lone newlines between blocks
		if (tokens[i].kind === LexTokenKind.BlankLine || tokens[i].kind === LexTokenKind.Newline) {
			i++;
			continue;
		}

		const result = parseBlockAt(tokens, i, source);
		if (result.node) {
			nodes.push(result.node);
			if (result.region) regions.push(result.region);
		}
		i = result.next;
	}

	return { nodes, regions };
}

interface BlockResult {
	node: AstNode | null;
	next: number;
	region?: RegionBoundary;
}

function parseBlockAt(tokens: LexToken[], i: number, source: string): BlockResult {
	const token = tokens[i];

	switch (token.kind) {
		case LexTokenKind.HeadingMarker:
			return parseHeading(tokens, i);

		case LexTokenKind.FenceOpen:
			return parseCodeBlock(tokens, i);

		case LexTokenKind.LatexBlockOpen:
			return parseLatexBlock(tokens, i);

		case LexTokenKind.HorizontalRule:
			return parseHorizontalRule(tokens, i);

		case LexTokenKind.BlockquoteMarker:
			return parseBlockquote(tokens, i, source);

		case LexTokenKind.ListItemMarker:
			return parseList(tokens, i);

		case LexTokenKind.TablePipe:
			return parseTable(tokens, i, source);

		case LexTokenKind.Text:
			// Could be start of a table row (text before first pipe) or a paragraph
			if (hasTablePipeOnSameLine(tokens, i)) {
				return parseTable(tokens, i, source);
			}
			return parseParagraph(tokens, i);

		default:
			// Unknown token — skip
			return { node: null, next: i + 1 };
	}
}

function hasTablePipeOnSameLine(tokens: LexToken[], i: number): boolean {
	// Look ahead to see if there's a TablePipe before the next Newline/BlankLine
	for (let j = i; j < tokens.length; j++) {
		const kind = tokens[j].kind;
		if (kind === LexTokenKind.TablePipe) return true;
		if (kind === LexTokenKind.Newline || kind === LexTokenKind.BlankLine) return false;
	}
	return false;
}

function parseHeading(tokens: LexToken[], i: number): BlockResult {
	const marker = tokens[i];
	const level = parseInt(marker.value);
	const start = marker.start;

	i++; // skip marker

	// Collect text content on this line
	let textContent = '';
	let textStart = marker.end;
	let end = marker.end;

	while (i < tokens.length) {
		const t = tokens[i];
		if (t.kind === LexTokenKind.Newline || t.kind === LexTokenKind.BlankLine) {
			end = t.end;
			i++;
			break;
		}
		if (t.kind === LexTokenKind.Text) {
			if (!textContent) textStart = t.start;
			textContent += t.value;
			end = t.end;
		}
		i++;
	}

	const children = textContent.length > 0 ? buildInlineAst(textContent, textStart) : [];

	const node: HeadingNode = {
		type: AstNodeType.Heading,
		level,
		start,
		end,
		children
	};

	return { node, next: i };
}

function parseCodeBlock(tokens: LexToken[], i: number): BlockResult {
	const openToken = tokens[i];
	const language = openToken.value || undefined;
	const start = openToken.start;

	i++; // skip FenceOpen

	let content = '';
	let closed = false;
	let end = openToken.end;

	while (i < tokens.length) {
		const t = tokens[i];
		if (t.kind === LexTokenKind.FenceClose) {
			end = t.end;
			closed = true;
			i++;
			break;
		}
		// Everything inside is content (Text tokens)
		content += t.value;
		end = t.end;
		i++;
	}

	// Trim trailing newline from content
	content = content.replace(/\n$/, '');

	const node: CodeBlockNode = {
		type: AstNodeType.CodeBlock,
		language,
		content,
		closed,
		start,
		end
	};

	const region: RegionBoundary = { type: 'codeblock', start, end };
	return { node, next: i, region };
}

function parseLatexBlock(tokens: LexToken[], i: number): BlockResult {
	const openToken = tokens[i];
	const start = openToken.start;

	i++; // skip LatexBlockOpen

	let content = '';
	let end = openToken.end;

	while (i < tokens.length) {
		const t = tokens[i];
		if (t.kind === LexTokenKind.LatexBlockClose) {
			end = t.end;
			i++;
			break;
		}
		content += t.value;
		end = t.end;
		i++;
	}

	const node: LatexBlockNode = {
		type: AstNodeType.LatexBlock,
		content: content.trim(),
		start,
		end
	};

	return { node, next: i };
}

function parseHorizontalRule(tokens: LexToken[], i: number): BlockResult {
	const token = tokens[i];
	const node: HorizontalRuleNode = {
		type: AstNodeType.HorizontalRule,
		start: token.start,
		end: token.end
	};
	return { node, next: i + 1 };
}

function parseBlockquote(tokens: LexToken[], i: number, source: string): BlockResult {
	const start = tokens[i].start;
	const lines: string[] = [];
	let end = tokens[i].end;

	while (i < tokens.length) {
		if (tokens[i].kind !== LexTokenKind.BlockquoteMarker) break;

		i++; // skip marker

		// Collect text on this line
		let lineContent = '';
		while (i < tokens.length) {
			const t = tokens[i];
			if (t.kind === LexTokenKind.Newline) {
				end = t.end;
				i++;
				break;
			}
			if (t.kind === LexTokenKind.BlankLine) {
				end = t.end;
				i++;
				// Blank line ends blockquote
				break;
			}
			if (t.kind === LexTokenKind.Text) {
				lineContent += t.value;
			}
			end = t.end;
			i++;
		}
		lines.push(lineContent);

		// Check if blank line follows (end of blockquote)
		if (i < tokens.length && tokens[i].kind === LexTokenKind.BlankLine) {
			end = tokens[i].end;
			i++;
			break;
		}
	}

	// Recursively parse blockquote content
	const innerSource = lines.join('\n');
	const innerResult = parseSync(innerSource);

	const node: BlockquoteNode = {
		type: AstNodeType.Blockquote,
		start,
		end,
		children: innerResult.nodes
	};

	const region: RegionBoundary = { type: 'blockquote', start, end };
	return { node, next: i, region };
}

function parseList(tokens: LexToken[], i: number): BlockResult {
	const start = tokens[i].start;
	const firstMarker = tokens[i].value;

	// Determine if ordered or unordered
	const isOrdered = /^\d+\.\s/.test(firstMarker);
	const startNumber = isOrdered ? parseInt(firstMarker.trim()) : undefined;

	const items: ListItemNode[] = [];
	let end = tokens[i].end;

	while (i < tokens.length && tokens[i].kind === LexTokenKind.ListItemMarker) {
		const markerToken = tokens[i];
		const currentMarker = markerToken.value;

		// Check consistency: don't mix ordered and unordered
		const currentIsOrdered = /^\d+\.\s/.test(currentMarker);
		if (currentIsOrdered !== isOrdered) break;

		const itemStart = markerToken.start;
		i++; // skip marker

		// Collect text content for this list item
		let textContent = '';
		let textStart = markerToken.end;
		let itemEnd = markerToken.end;

		while (i < tokens.length) {
			const t = tokens[i];
			if (t.kind === LexTokenKind.Newline) {
				itemEnd = t.end;
				i++;
				break;
			}
			if (t.kind === LexTokenKind.BlankLine) {
				itemEnd = t.end;
				i++;
				break;
			}
			if (t.kind === LexTokenKind.ListItemMarker) {
				break;
			}
			if (t.kind === LexTokenKind.Text) {
				if (!textContent) textStart = t.start;
				textContent += t.value;
			}
			itemEnd = t.end;
			i++;
		}

		const children = textContent.length > 0 ? buildInlineAst(textContent, textStart) : [];

		const item: ListItemNode = {
			type: AstNodeType.ListItem,
			start: itemStart,
			end: itemEnd,
			children
		};
		items.push(item);
		end = itemEnd;

		// Stop on blank line
		if (i > 0 && tokens[i - 1]?.kind === LexTokenKind.BlankLine) {
			break;
		}
	}

	if (items.length === 0) {
		return { node: null, next: i };
	}

	const node: AstNode = isOrdered
		? ({
				type: AstNodeType.OrderedList,
				startNumber,
				start,
				end,
				children: items
			} as OrderedListNode)
		: ({
				type: AstNodeType.UnorderedList,
				start,
				end,
				children: items
			} as UnorderedListNode);

	const region: RegionBoundary = { type: 'list', start, end };
	return { node, next: i, region };
}

function parseTable(tokens: LexToken[], i: number, source: string): BlockResult {
	const start = tokens[i].start;

	// Collect all lines that form part of the table
	const tableLines: { tokens: LexToken[]; lineStart: number; lineEnd: number }[] = [];

	// Read first line
	const firstLine = collectTableLine(tokens, i);
	if (!firstLine) {
		return parseParagraph(tokens, i);
	}
	tableLines.push(firstLine);
	i = firstLine.nextIdx;

	// Skip newline/blank between first line and separator
	while (i < tokens.length && tokens[i].kind === LexTokenKind.Newline) i++;

	// Check for separator row
	if (i < tokens.length && tokens[i].kind === LexTokenKind.TableSeparatorRow) {
		i++; // skip separator
		// Skip newline after separator
		while (i < tokens.length && tokens[i].kind === LexTokenKind.Newline) i++;
	} else {
		// Not a valid table — parse as paragraph instead
		return parseParagraph(tokens, start === tokens[0]?.start ? 0 : findTokenIndex(tokens, start));
	}

	// Read remaining data rows
	while (i < tokens.length) {
		const t = tokens[i];
		if (t.kind === LexTokenKind.BlankLine) {
			i++;
			break;
		}

		const isTableContent = t.kind === LexTokenKind.TablePipe || t.kind === LexTokenKind.Text;
		if (!isTableContent) break;

		// Check if this line has pipes (is a table row)
		if (!hasTablePipeOnSameLine(tokens, i) && t.kind !== LexTokenKind.TablePipe) {
			break;
		}

		const line = collectTableLine(tokens, i);
		if (!line) break;
		tableLines.push(line);
		i = line.nextIdx;

		// Skip trailing newlines
		while (i < tokens.length && tokens[i].kind === LexTokenKind.Newline) i++;
	}

	if (tableLines.length === 0) {
		return { node: null, next: i };
	}

	// Build table rows
	const rows: TableRowNode[] = [];
	let end = start;

	for (let lineIdx = 0; lineIdx < tableLines.length; lineIdx++) {
		const line = tableLines[lineIdx];
		const isHeader = lineIdx === 0;
		const cells = buildTableCells(line.tokens);

		const row: TableRowNode = {
			type: AstNodeType.TableRow,
			isHeader,
			start: line.lineStart,
			end: line.lineEnd,
			children: cells
		};
		rows.push(row);
		end = line.lineEnd;
	}

	const node: TableNode = {
		type: AstNodeType.Table,
		start,
		end,
		children: rows
	};

	const region: RegionBoundary = { type: 'table', start, end };
	return { node, next: i, region };
}

interface TableLineResult {
	tokens: LexToken[];
	lineStart: number;
	lineEnd: number;
	nextIdx: number;
}

function collectTableLine(tokens: LexToken[], i: number): TableLineResult | null {
	if (i >= tokens.length) return null;

	const lineTokens: LexToken[] = [];
	const lineStart = tokens[i].start;
	let lineEnd = tokens[i].end;

	while (i < tokens.length) {
		const t = tokens[i];
		if (t.kind === LexTokenKind.Newline || t.kind === LexTokenKind.BlankLine) {
			lineEnd = t.start; // end before the newline
			i++;
			break;
		}
		lineTokens.push(t);
		lineEnd = t.end;
		i++;
	}

	if (lineTokens.length === 0) return null;

	return { tokens: lineTokens, lineStart, lineEnd, nextIdx: i };
}

function buildTableCells(lineTokens: LexToken[]): TableCellNode[] {
	const cells: TableCellNode[] = [];

	// Split on TablePipe tokens
	let cellTokens: LexToken[] = [];
	let cellStart = lineTokens.length > 0 ? lineTokens[0].start : 0;

	for (const token of lineTokens) {
		if (token.kind === LexTokenKind.TablePipe) {
			// If we have content, build a cell
			if (cellTokens.length > 0) {
				const textContent = cellTokens
					.map((t) => t.value)
					.join('')
					.trim();
				const cellEnd = token.start;
				if (textContent.length > 0) {
					const children = buildInlineAst(textContent, cellTokens[0].start);
					cells.push({
						type: AstNodeType.TableCell,
						start: cellStart,
						end: cellEnd,
						children
					});
				}
			}
			cellTokens = [];
			cellStart = token.end;
		} else {
			cellTokens.push(token);
		}
	}

	// Handle content after last pipe
	if (cellTokens.length > 0) {
		const textContent = cellTokens
			.map((t) => t.value)
			.join('')
			.trim();
		if (textContent.length > 0) {
			const cellEnd = cellTokens[cellTokens.length - 1].end;
			const children = buildInlineAst(textContent, cellTokens[0].start);
			cells.push({
				type: AstNodeType.TableCell,
				start: cellStart,
				end: cellEnd,
				children
			});
		}
	}

	return cells;
}

function findTokenIndex(tokens: LexToken[], start: number): number {
	for (let i = 0; i < tokens.length; i++) {
		if (tokens[i].start >= start) return i;
	}
	return tokens.length;
}

function parseParagraph(tokens: LexToken[], i: number): BlockResult {
	const start = tokens[i].start;
	let textContent = '';
	let textStart = tokens[i].start;
	let end = tokens[i].end;

	while (i < tokens.length) {
		const t = tokens[i];

		if (t.kind === LexTokenKind.BlankLine) {
			break;
		}

		// Stop if we hit a block-level marker (except Newline)
		if (isBlockStart(t.kind)) {
			break;
		}

		if (t.kind === LexTokenKind.Text) {
			if (!textContent) textStart = t.start;
			textContent += t.value;
			end = t.end;
		} else if (t.kind === LexTokenKind.Newline) {
			// Single newline within paragraph — continue, add a linebreak
			textContent += '\n';
			end = t.end;
		}

		i++;
	}

	if (textContent.trim().length === 0) {
		return { node: null, next: i === 0 ? i + 1 : i };
	}

	const children = buildInlineAst(textContent.trimEnd(), textStart);

	const node: ParagraphNode = {
		type: AstNodeType.Paragraph,
		start,
		end,
		children
	};

	const region: RegionBoundary = { type: 'paragraph', start, end };
	return { node, next: i, region };
}

function isBlockStart(kind: LexTokenKind): boolean {
	return (
		kind === LexTokenKind.HeadingMarker ||
		kind === LexTokenKind.FenceOpen ||
		kind === LexTokenKind.HorizontalRule ||
		kind === LexTokenKind.BlockquoteMarker ||
		kind === LexTokenKind.ListItemMarker ||
		kind === LexTokenKind.LatexBlockOpen ||
		kind === LexTokenKind.TableSeparatorRow
	);
}
