export { AstNodeType } from './types';
export type {
	AstNode,
	TextNode,
	HeadingNode,
	ParagraphNode,
	CodeBlockNode,
	BlockquoteNode,
	OrderedListNode,
	UnorderedListNode,
	ListItemNode,
	TableNode,
	TableRowNode,
	TableCellNode,
	HorizontalRuleNode,
	LatexBlockNode,
	LatexInlineNode,
	BoldNode,
	ItalicNode,
	StrikethroughNode,
	InlineCodeNode,
	LinkNode,
	ImageNode,
	LineBreakNode,
	ParseResult,
	RegionBoundary
} from './types';

import type { ParseResult } from './types';

/**
 * Synchronous parse — used by the web worker and incremental module.
 * Eagerly imports the block parser.
 */
export { parseSync } from './block';

/**
 * Async parse via dynamic import() for code splitting.
 * The block parser chunk is only loaded when first called.
 */
export async function parseAsync(source: string): Promise<ParseResult> {
	const { parseSync } = await import('./block');
	return parseSync(source);
}
