/**
 * Custom markdown batch parser
 *
 * Features:
 * - Batch parsing with fake incremental support via region boundaries
 * - LaTeX support with relaxed spacing rules
 * - Non-standard features (tables with tabs, custom citations)
 * - Simple, clear token structure
 */

export { parse, MarkdownParser } from './parser';
export type { ParserOptions } from './parser';
import { parse, MarkdownParser } from './parser';
import type { Token, ParseResult } from './tokens';

export {
	TokenType,
	type Token,
	type TextToken,
	type HeadingToken,
	type ParagraphToken,
	type CodeBlockToken,
	type BlockquoteToken,
	type OrderedListToken,
	type UnorderedListToken,
	type ListItemToken,
	type TableToken,
	type TableRowToken,
	type TableCellToken,
	type HorizontalRuleToken,
	type LatexBlockToken,
	type LatexInlineToken,
	type BoldToken,
	type ItalicToken,
	type StrikethroughToken,
	type InlineCodeToken,
	type LinkToken,
	type ImageToken,
	type CitationToken,
	type LineBreakToken,
	type ParseResult,
	type RegionBoundary
} from './tokens';

/**
 * Incremental parsing state
 */
export interface IncrementalState {
	/** Previous source text */
	prevSource: string;
	/** Previous parse result */
	prevResult: ParseResult;
	/** Position where new content starts */
	newContentStart: number;
}

/**
 * Result of incremental parsing attempt
 */
export interface IncrementalParseResult {
	/** The updated parse result */
	result: ParseResult;
	/** New state for next incremental parse */
	state: IncrementalState;
}

/**
 * Async wrapper for full parse (for code splitting)
 */
export async function parseAsync(source: string): Promise<ParseResult> {
	return parse(source);
}

/**
 * Pure function for incremental parsing
 * Detects region boundaries and attempts to reuse previous tokens
 *
 * @param source - Current source text
 * @param state - Previous parsing state (if available)
 * @returns Parse result and new state
 */
export async function parseIncremental(
	source: string,
	state: IncrementalState | null
): Promise<IncrementalParseResult> {
	// If no previous state or source doesn't extend previous, do full parse
	if (!state || !source.startsWith(state.prevSource)) {
		const result = parse(source);
		return {
			result,
			state: {
				prevSource: source,
				prevResult: result,
				newContentStart: source.length
			}
		};
	}

	// Check if new content is just whitespace
	const newContent = source.slice(state.prevSource.length);
	if (newContent.trim().length === 0) {
		// No meaningful new content, return previous result
		return {
			result: state.prevResult,
			state: {
				prevSource: source,
				prevResult: state.prevResult,
				newContentStart: source.length
			}
		};
	}

	// Check if new content might change interpretation of previous content
	// This handles the case where adding new lines converts previous paragraphs to tables, lists, etc.
	const needsReparse = shouldReparseFromEarlier(state.prevResult, source, state.prevSource);

	if (needsReparse) {
		// Need to re-parse more aggressively
		const result = parse(source);
		return {
			result,
			state: {
				prevSource: source,
				prevResult: result,
				newContentStart: source.length
			}
		};
	}

	// Find the last complete region in previous result
	const lastCompleteRegionEnd = findLastCompleteRegion(state.prevResult, state.prevSource.length);

	if (lastCompleteRegionEnd === 0) {
		// No complete regions found, do full parse
		const result = parse(source);
		return {
			result,
			state: {
				prevSource: source,
				prevResult: result,
				newContentStart: source.length
			}
		};
	}

	// Get tokens that are before the last complete region
	const stableTokens = state.prevResult.tokens.filter(
		(token: Token) => token.end <= lastCompleteRegionEnd
	);

	// Parse only the new content from the last complete region
	const newContentToParse = source.slice(lastCompleteRegionEnd);
	const parser = new MarkdownParser(newContentToParse);
	const newParseResult = parser.parse();

	// Adjust positions of new tokens
	const adjustedNewTokens = newParseResult.tokens.map((token) =>
		adjustTokenPosition(token, lastCompleteRegionEnd)
	);

	// Adjust region boundaries
	const adjustedNewRegions = newParseResult.regions.map((region) => ({
		...region,
		start: region.start + lastCompleteRegionEnd,
		end: region.end + lastCompleteRegionEnd
	}));

	// Combine stable and new tokens
	const combinedTokens = [...stableTokens, ...adjustedNewTokens];
	const combinedRegions = [
		...state.prevResult.regions.filter((r: { end: number }) => r.end <= lastCompleteRegionEnd),
		...adjustedNewRegions
	];

	const result: ParseResult = {
		tokens: combinedTokens,
		regions: combinedRegions
	};

	return {
		result,
		state: {
			prevSource: source,
			prevResult: result,
			newContentStart: source.length
		}
	};
}

/**
 * Check if new content might change the interpretation of previous content
 * This handles cases where adding new lines should trigger re-parsing from earlier
 * 
 * Examples:
 * - Adding a table separator line after a line with pipes
 * - Adding a list marker after a potential list item
 * - Adding closing fence for code blocks
 */
function shouldReparseFromEarlier(
	prevResult: ParseResult,
	newSource: string,
	oldSource: string
): boolean {
	if (prevResult.tokens.length === 0) {
		return false;
	}

	const newContent = newSource.slice(oldSource.length);
	const lastToken = prevResult.tokens[prevResult.tokens.length - 1];

	// Check if last token is a paragraph that ends at the old source boundary
	// and new content might convert it to a table
	if (lastToken.type === 'Paragraph' && lastToken.end >= oldSource.length - 2) {
		// Get the last line of old source
		const lastLineStart = oldSource.lastIndexOf('\n', oldSource.length - 2) + 1;
		const lastLine = oldSource.slice(lastLineStart).trim();

		// Check if last line looks like a table row (has pipes or tabs)
		const hasPipes = lastLine.includes('|');
		const hasTabs = lastLine.includes('\t');

		if (hasPipes || hasTabs) {
			// Check if new content starts with a table separator
			const newLines = newContent.trim().split('\n');
			const firstNewLine = newLines[0];

			// Table separator pattern: |---|---| or ---\t---
			if (firstNewLine && /^[\|\s]*:?-+:?[\|\s:]*(:?-+:?[\|\s]*)*$/.test(firstNewLine.trim())) {
				return true;
			}
		}
	}

	// Check if we're building a table incrementally (row by row)
	// If the last token is a table or paragraph ending near the boundary,
	// and new content has table-like structure, we might need to re-parse
	if (lastToken.end >= oldSource.length - 2) {
		const newLines = newContent.trim().split('\n');
		if (newLines.length > 0) {
			const firstNewLine = newLines[0].trim();

			// If new line looks like a table row or separator
			const looksLikeTableRow = firstNewLine.includes('|') || firstNewLine.includes('\t');
			const looksLikeSeparator = /^[\|\s]*:?-+:?[\|\s:]*(:?-+:?[\|\s]*)*$/.test(firstNewLine);

			if (looksLikeTableRow || looksLikeSeparator) {
				// Check if previous content ends with something table-like
				const prevLines = oldSource.trim().split('\n');
				const lastPrevLine = prevLines[prevLines.length - 1];

				if (
					lastPrevLine &&
					(lastPrevLine.includes('|') || lastPrevLine.includes('\t') || /^[\|\s]*:?-+:?/.test(lastPrevLine))
				) {
					return true;
				}
			}
		}
	}

	return false;
}

/**
 * Find the end position of the last complete region
 * A region is complete if it has proper closing (e.g., blank line after paragraph)
 */
function findLastCompleteRegion(result: ParseResult, sourceLength: number): number {
	if (result.tokens.length === 0) {
		return 0;
	}

	// Use region boundaries for better detection
	// Regions like tables, lists, blockquotes, code blocks have clear boundaries
	if (result.regions.length > 0) {
		// Find the last complete region (not at the very end)
		for (let i = result.regions.length - 1; i >= 0; i--) {
			const region = result.regions[i];
			if (region.end < sourceLength - 1) {
				return region.end;
			}
		}
	}

	// Fallback: Look for tokens that are definitely complete
	// A token is complete if it ends before the current source end
	// and is followed by whitespace or another token
	for (let i = result.tokens.length - 2; i >= 0; i--) {
		const token = result.tokens[i];
		const nextToken = result.tokens[i + 1];

		// Check if this token is well-separated from the end
		if (token.end < sourceLength - 2 && nextToken && nextToken.start > token.end) {
			return token.end;
		}
	}

	// Last resort: if we have multiple tokens, use the second-to-last
	if (result.tokens.length > 1) {
		return result.tokens[result.tokens.length - 2].end;
	}

	// No complete regions found
	return 0;
}

/**
 * Recursively adjust token positions by an offset
 */
function adjustTokenPosition(token: Token, offset: number): Token {
	const adjusted = {
		...token,
		start: token.start + offset,
		end: token.end + offset
	};

	if (token.children) {
		adjusted.children = token.children.map((child: Token) => adjustTokenPosition(child, offset));
	}

	return adjusted;
}
