/**
 * Custom markdown batch parser
 *
 * Features:
 * - Batch parsing with fake incremental support via region boundaries
 * - LaTeX support with relaxed spacing rules
 * - Non-standard features (tables with tabs)
 * - Simple, clear token structure
 */

import { parseBlocks } from './parsers/block-parser';
import { parseInline } from './parsers/inline-parser';
import { parseIncremental, type IncrementalState, type IncrementalParseResult } from './parsers/incremental';
import { TokenType } from './tokens';
import type {
	Token,
	TextToken,
	HeadingToken,
	ParagraphToken,
	CodeBlockToken,
	BlockquoteToken,
	OrderedListToken,
	UnorderedListToken,
	ListItemToken,
	TableToken,
	TableRowToken,
	TableCellToken,
	HorizontalRuleToken,
	LatexBlockToken,
	LatexInlineToken,
	BoldToken,
	ItalicToken,
	StrikethroughToken,
	InlineCodeToken,
	LinkToken,
	ImageToken,
	LineBreakToken,
	ParseResult,
	RegionBoundary
} from './tokens';

export { parseInline };
export { parseBlocks };
export { parseIncremental };
export { TokenType };
export type {
	IncrementalState,
	IncrementalParseResult,
	Token,
	TextToken,
	HeadingToken,
	ParagraphToken,
	CodeBlockToken,
	BlockquoteToken,
	OrderedListToken,
	UnorderedListToken,
	ListItemToken,
	TableToken,
	TableRowToken,
	TableCellToken,
	HorizontalRuleToken,
	LatexBlockToken,
	LatexInlineToken,
	BoldToken,
	ItalicToken,
	StrikethroughToken,
	InlineCodeToken,
	LinkToken,
	ImageToken,
	LineBreakToken,
	ParseResult,
	RegionBoundary
};

export interface ParserOptions {
	startFrom?: number;
}

export function parse(source: string, options?: ParserOptions): ParseResult {
	const startPosition = options?.startFrom || 0;
	const { tokens, regions } = parseBlocks(source, startPosition);

	return {
		tokens,
		regions
	};
}

export async function parseAsync(source: string): Promise<ParseResult> {
	return parse(source);
}
