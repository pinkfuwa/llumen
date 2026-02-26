import { TokenType, type Token, type RegionBoundary } from '../tokens';
import * as builders from '../tokens/builders';
import { parseInline } from './inline-parser';

export interface ParseContext {
	source: string;
	position: number;
}

export interface BlockParseResult {
	token: Token | null;
	newPosition: number;
	regions: RegionBoundary[];
}

export function peekLine(source: string, position: number): string {
	const newlinePos = source.indexOf('\n', position);
	if (newlinePos === -1) {
		return source.substring(position);
	}
	return source.substring(position, newlinePos);
}

export function peek(source: string, position: number, n: number): string {
	return source.substring(position, position + n);
}

export function skipWhitespace(source: string, position: number): number {
	let pos = position;
	while (pos < source.length && (source[pos] === ' ' || source[pos] === '\t')) {
		pos++;
	}
	return pos;
}

export function skipNewlines(source: string, position: number): { newPosition: number; hadBlankLine: boolean } {
	let newlineCount = 0;
	let pos = position;
	while (pos < source.length && (source[pos] === '\n' || source[pos] === '\r')) {
		if (source[pos] === '\n') {
			newlineCount++;
		}
		pos++;
	}
	return { newPosition: pos, hadBlankLine: newlineCount >= 2 };
}

export function isTableRow(line: string): boolean {
	if (line.includes('`')) {
		return false;
	}
	const trimmed = line.trim();
	const pipeCount = (trimmed.match(/\|/g) || []).length;
	return pipeCount >= 2 || trimmed.includes('\t');
}

export function isTableSeparator(line: string): boolean {
	const trimmed = line.trim();
	return /^[\|\s]*:?-+:?[\|\s:]*(:?-+:?[\|\s]*)*$/.test(trimmed);
}

export function parseHeading(ctx: ParseContext): BlockParseResult {
	const start = ctx.position;
	const line = peekLine(ctx.source, ctx.position);

	const match = line.match(/^(#{1,6})\s+(.+)$/);
	if (!match) {
		return { token: null, newPosition: ctx.position, regions: [] };
	}

	const level = match[1].length;
	const content = match[2];

	let newPos = ctx.position + line.length;
	const skipResult = skipNewlines(ctx.source, newPos);
	newPos = skipResult.newPosition;

	const inlineTokens = parseInline(content, start + match[1].length + 1);

	return {
		token: builders.createHeadingToken(level, inlineTokens, start, newPos),
		newPosition: newPos,
		regions: []
	};
}

export function parseCodeBlock(ctx: ParseContext): BlockParseResult {
	const start = ctx.position;
	const line = peekLine(ctx.source, ctx.position);

	const match = line.match(/^\s*```(\w*)$/);
	if (!match) {
		return { token: null, newPosition: ctx.position, regions: [] };
	}

	const language = match[1] || undefined;
	let newPos = ctx.position + line.length;
	const skipResult = skipNewlines(ctx.source, newPos);
	newPos = skipResult.newPosition;

	const contentStart = newPos;
	let contentEnd = newPos;
	let closed = false;

	while (newPos < ctx.source.length) {
		const currentLine = peekLine(ctx.source, newPos);
		if (currentLine.match(/^\s*```\s*$/)) {
			contentEnd = newPos;
			newPos += currentLine.length;
			const skip = skipNewlines(ctx.source, newPos);
			newPos = skip.newPosition;
			closed = true;
			break;
		}
		newPos += currentLine.length;
		const skip = skipNewlines(ctx.source, newPos);
		newPos = skip.newPosition;
	}

	if (contentEnd === contentStart && newPos >= ctx.source.length) {
		contentEnd = newPos;
	}

	const content = ctx.source.substring(contentStart, contentEnd).trimEnd();

	return {
		token: builders.createCodeBlockToken(language, content, closed, start, newPos),
		newPosition: newPos,
		regions: [
			{
				type: 'codeblock',
				start,
				end: newPos
			}
		]
	};
}

export function parseLatexBlock(ctx: ParseContext): BlockParseResult {
	const originalPosition = ctx.position;
	const originalStart = ctx.position;

	let pos = skipWhitespace(ctx.source, ctx.position);
	let delimiter: string;
	let endDelimiter: string;

	if (peek(ctx.source, pos, 2) === '\\[') {
		delimiter = '\\[';
		endDelimiter = '\\]';
	} else if (peek(ctx.source, pos, 2) === '$$') {
		delimiter = '$$';
		endDelimiter = '$$';
	} else {
		const line = peekLine(ctx.source, ctx.position);
		const bracketPos = line.indexOf('\\[');
		if (bracketPos !== -1) {
			pos = ctx.position + bracketPos;
			delimiter = '\\[';
			endDelimiter = '\\]';
		} else {
			return { token: null, newPosition: ctx.position, regions: [] };
		}
	}

	const afterDelimiter = peek(ctx.source, pos, delimiter.length + 1);
	if (delimiter === '$$' && afterDelimiter[delimiter.length] !== '\n') {
		return { token: null, newPosition: ctx.position, regions: [] };
	}

	pos += delimiter.length;
	if (delimiter === '$$') {
		const skip = skipNewlines(ctx.source, pos);
		pos = skip.newPosition;
	}

	const contentStart = pos;
	const endPos = ctx.source.indexOf(endDelimiter, pos);

	if (endPos === -1) {
		return { token: null, newPosition: ctx.position, regions: [] };
	}

	const content = ctx.source.substring(contentStart, endPos);
	pos = endPos + endDelimiter.length;
	const skip = skipNewlines(ctx.source, pos);
	pos = skip.newPosition;

	return {
		token: builders.createLatexBlockToken(content.trim(), originalStart, pos),
		newPosition: pos,
		regions: []
	};
}

export function parseHorizontalRule(ctx: ParseContext): BlockParseResult {
	const start = ctx.position;
	const line = peekLine(ctx.source, ctx.position);

	if (line.match(/^---+$/) || line.match(/^\*\*\*+$/) || line.match(/^___+$/)) {
		const newPos = ctx.position + line.length;
		const skip = skipNewlines(ctx.source, newPos);
		return {
			token: builders.createHorizontalRuleToken(start, skip.newPosition),
			newPosition: skip.newPosition,
			regions: []
		};
	}

	return { token: null, newPosition: ctx.position, regions: [] };
}

export function parseTable(ctx: ParseContext): BlockParseResult {
	const start = ctx.position;
	const firstLine = peekLine(ctx.source, ctx.position);

	if (!isTableRow(firstLine)) {
		return { token: null, newPosition: ctx.position, regions: [] };
	}

	let savedPosition = ctx.position + firstLine.length;
	let skip = skipNewlines(ctx.source, savedPosition);
	const secondLine = peekLine(ctx.source, skip.newPosition);

	if (!isTableSeparator(secondLine)) {
		return { token: null, newPosition: ctx.position, regions: [] };
	}

	const rows: Token[] = [];
	let lineNum = 0;
	let pos = ctx.position;

	while (pos < ctx.source.length) {
		const line = peekLine(ctx.source, pos);

		if (lineNum === 1 && isTableSeparator(line)) {
			pos += line.length;
			skip = skipNewlines(ctx.source, pos);
			pos = skip.newPosition;
			lineNum++;
			continue;
		}

		if (!isTableRow(line)) {
			break;
		}

		const rowStart = pos;
		const cells = parseTableRow(ctx.source, line, rowStart);
		pos += line.length;
		skip = skipNewlines(ctx.source, pos);
		pos = skip.newPosition;

		rows.push(
			builders.createTableRowToken(lineNum === 0, cells, rowStart, pos)
		);

		lineNum++;

		if (skip.hadBlankLine) {
			break;
		}
	}

	if (rows.length < 1) {
		return { token: null, newPosition: ctx.position, regions: [] };
	}

	return {
		token: builders.createTableToken(rows, start, pos),
		newPosition: pos,
		regions: [
			{
				type: 'table',
				start,
				end: pos
			}
		]
	};
}

function parseTableRow(source: string, line: string, rowStart: number): Token[] {
	const cells: Token[] = [];
	const trimmed = line.trim();

	let parts: string[];
	if (trimmed.includes('|')) {
		parts = trimmed.split('|').map((p) => p.trim());
		if (parts[0] === '') parts.shift();
		if (parts[parts.length - 1] === '') parts.pop();
	} else {
		parts = trimmed.split('\t').map((p) => p.trim());
	}

	let offset = rowStart;
	for (const part of parts) {
		const cellStart = offset;
		const inlineTokens = parseInline(part, cellStart);
		offset += part.length + 1;

		cells.push(
			builders.createTableCellToken(undefined, inlineTokens, cellStart, offset)
		);
	}

	return cells;
}

export function parseBlockquote(ctx: ParseContext): BlockParseResult {
	const start = ctx.position;
	const line = peekLine(ctx.source, ctx.position);

	if (!line.startsWith('>')) {
		return { token: null, newPosition: ctx.position, regions: [] };
	}

	const lines: string[] = [];
	let pos = ctx.position;

	while (pos < ctx.source.length) {
		const currentLine = peekLine(ctx.source, pos);
		if (!currentLine.startsWith('>')) {
			break;
		}

		lines.push(currentLine.substring(1).trim());
		pos += currentLine.length;
		const skip = skipNewlines(ctx.source, pos);
		pos = skip.newPosition;

		if (skip.hadBlankLine) {
			break;
		}
	}

	const content = lines.join('\n');

	const { tokens } = parseBlocks(content, 0);

	return {
		token: builders.createBlockquoteToken(tokens, start, pos),
		newPosition: pos,
		regions: [
			{
				type: 'blockquote',
				start,
				end: pos
			}
		]
	};
}

export function parseList(ctx: ParseContext): BlockParseResult {
	const start = ctx.position;
	const line = peekLine(ctx.source, ctx.position);

	const orderedMatch = line.match(/^(\d+)\.\s+/);
	const unorderedMatch = line.match(/^[-*+]\s+/);

	if (!orderedMatch && !unorderedMatch) {
		return { token: null, newPosition: ctx.position, regions: [] };
	}

	const isOrdered = !!orderedMatch;
	const startNumber = orderedMatch ? parseInt(orderedMatch[1]) : undefined;
	const items: Token[] = [];
	let pos = ctx.position;

	while (pos < ctx.source.length) {
		const currentLine = peekLine(ctx.source, pos);
		const itemMatch = isOrdered
			? currentLine.match(/^(\d+)\.\s+(.*)$/)
			: currentLine.match(/^[-*+]\s+(.*)$/);

		if (!itemMatch) {
			break;
		}

		const itemStart = pos;
		const itemContent = isOrdered ? itemMatch[2] : itemMatch[1];

		pos += currentLine.length;
		const skip = skipNewlines(ctx.source, pos);
		pos = skip.newPosition;

		const inlineTokens = parseInline(
			itemContent,
			itemStart + (currentLine.length - itemContent.length)
		);

		items.push(
			builders.createListItemToken(inlineTokens, itemStart, pos)
		);

		if (skip.hadBlankLine) {
			break;
		}
	}

	if (items.length === 0) {
		return { token: null, newPosition: ctx.position, regions: [] };
	}

	if (isOrdered) {
		return {
			token: builders.createOrderedListToken(startNumber, items, start, pos),
			newPosition: pos,
			regions: [
				{
					type: 'list',
					start,
					end: pos
				}
			]
		};
	} else {
		return {
			token: builders.createUnorderedListToken(items, start, pos),
			newPosition: pos,
			regions: [
				{
					type: 'list',
					start,
					end: pos
				}
			]
		};
	}
}

export function parseIndentedList(ctx: ParseContext): BlockParseResult {
	const start = ctx.position;
	const line = peekLine(ctx.source, ctx.position);

	const indentedMatch = line.match(/^\s+[-*+]\s+/);
	if (!indentedMatch) {
		return { token: null, newPosition: ctx.position, regions: [] };
	}

	const items: Token[] = [];
	let pos = ctx.position;

	while (pos < ctx.source.length) {
		const currentLine = peekLine(ctx.source, pos);
		const itemMatch = currentLine.match(/^\s+([-*+])\s+(.*)$/);

		if (!itemMatch) {
			const emptyLineMatch = currentLine.match(/^(\s*)$/);
			if (emptyLineMatch) {
				pos += currentLine.length;
				break;
			}
			break;
		}

		const itemStart = pos;
		const itemContent = itemMatch[2];

		pos += currentLine.length;
		const skip = skipNewlines(ctx.source, pos);
		pos = skip.newPosition;

		const inlineTokens = parseInline(
			itemContent,
			itemStart + (currentLine.length - itemContent.length)
		);

		items.push(
			builders.createListItemToken(inlineTokens, itemStart, pos)
		);

		if (skip.hadBlankLine) {
			break;
		}
	}

	if (items.length === 0) {
		return { token: null, newPosition: ctx.position, regions: [] };
	}

	return {
		token: builders.createUnorderedListToken(items, start, pos),
		newPosition: pos,
		regions: [
			{
				type: 'list',
				start,
				end: pos
			}
		]
	};
}

export function looksLikeBlockStart(line: string): boolean {
	return (
		line.match(/^#{1,6}\s/) !== null ||
		line.match(/^\s*```/) !== null ||
		line.match(/^(---+|\*\*\*+|___+)$/) !== null ||
		line.startsWith('>') ||
		line.match(/^\d+\.\s/) !== null ||
		line.match(/^[-*+]\s/) !== null ||
		line.match(/^\s*\\\[/) !== null ||
		line.match(/^\$\$/) !== null ||
		isTableRow(line)
	);
}

export function parseParagraph(ctx: ParseContext): BlockParseResult {
	const start = ctx.position;
	let pos = ctx.position;

	while (pos < ctx.source.length) {
		const line = peekLine(ctx.source, pos);

		if (line.trim() === '') {
			break;
		}

		if (looksLikeBlockStart(line)) {
			break;
		}

		pos += line.length;

		const nextChar = ctx.source[pos];
		if (nextChar === '\n' || nextChar === '\r') {
			pos++;
			if (nextChar === '\r' && ctx.source[pos] === '\n') {
				pos++;
			}
		}
	}

	if (pos === start) {
		return { token: null, newPosition: ctx.position, regions: [] };
	}

	const content = ctx.source.substring(start, pos).trimEnd();
	const inlineTokens = parseInline(content, start);

	return {
		token: builders.createParagraphToken(inlineTokens, start, pos),
		newPosition: pos,
		regions: [
			{
				type: 'paragraph',
				start,
				end: pos
			}
		]
	};
}

export function skipBlankLines(ctx: ParseContext): number {
	let pos = ctx.position;
	while (pos < ctx.source.length && (ctx.source[pos] === '\n' || ctx.source[pos] === '\r')) {
		pos++;
	}
	return pos;
}

export function parseBlock(ctx: ParseContext): BlockParseResult {
	const pos = skipBlankLines(ctx);
	ctx = { ...ctx, position: pos };

	if (ctx.position >= ctx.source.length) {
		return { token: null, newPosition: ctx.position, regions: [] };
	}

	const start = ctx.position;

	const heading = parseHeading(ctx);
	if (heading.token) return heading;

	const codeBlock = parseCodeBlock(ctx);
	if (codeBlock.token) return codeBlock;

	const latexBlock = parseLatexBlock(ctx);
	if (latexBlock.token) return latexBlock;

	const hr = parseHorizontalRule(ctx);
	if (hr.token) return hr;

	const table = parseTable(ctx);
	if (table.token) return table;

	const blockquote = parseBlockquote(ctx);
	if (blockquote.token) return blockquote;

	const list = parseList(ctx);
	if (list.token) return list;

	const indentedList = parseIndentedList(ctx);
	if (indentedList.token) return indentedList;

	const paragraph = parseParagraph(ctx);
	if (paragraph.token) return paragraph;

	return { token: null, newPosition: ctx.position + 1, regions: [] };
}

export function parseBlocks(source: string, startPosition: number = 0): { tokens: Token[]; regions: RegionBoundary[] } {
	const tokens: Token[] = [];
	const regions: RegionBoundary[] = [];

	let ctx: ParseContext = {
		source,
		position: startPosition
	};

	while (ctx.position < ctx.source.length) {
		const result = parseBlock(ctx);
		if (result.token) {
			tokens.push(result.token);
			regions.push(...result.regions);
		}
		ctx = { ...ctx, position: result.newPosition };
	}

	return { tokens, regions };
}
