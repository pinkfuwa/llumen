/**
 * Token types for markdown parsing
 */
export enum TokenType {
	// Block-level tokens
	Heading = 'Heading',
	Paragraph = 'Paragraph',
	CodeBlock = 'CodeBlock',
	Blockquote = 'Blockquote',
	OrderedList = 'OrderedList',
	UnorderedList = 'UnorderedList',
	ListItem = 'ListItem',
	Table = 'Table',
	TableRow = 'TableRow',
	TableCell = 'TableCell',
	TableHeader = 'TableHeader',
	HorizontalRule = 'HorizontalRule',
	LatexBlock = 'LatexBlock',

	// Inline tokens
	Text = 'Text',
	Bold = 'Bold',
	Italic = 'Italic',
	Strikethrough = 'Strikethrough',
	InlineCode = 'InlineCode',
	Link = 'Link',
	Image = 'Image',
	LatexInline = 'LatexInline',
	Citation = 'Citation',
	LineBreak = 'LineBreak'
}

/**
 * Base token interface
 */
export interface Token {
	type: TokenType;
	start: number;
	end: number;
	children?: Token[];
}

/**
 * Text token
 */
export interface TextToken extends Token {
	type: TokenType.Text;
	content: string;
}

/**
 * Heading token (h1-h6)
 */
export interface HeadingToken extends Token {
	type: TokenType.Heading;
	level: number; // 1-6
	children: Token[];
}

/**
 * Paragraph token
 */
export interface ParagraphToken extends Token {
	type: TokenType.Paragraph;
	children: Token[];
}

/**
 * Code block token
 */
export interface CodeBlockToken extends Token {
	type: TokenType.CodeBlock;
	language?: string;
	content: string;
}

/**
 * Blockquote token
 */
export interface BlockquoteToken extends Token {
	type: TokenType.Blockquote;
	children: Token[];
}

/**
 * List tokens
 */
export interface OrderedListToken extends Token {
	type: TokenType.OrderedList;
	startNumber?: number;
	children: ListItemToken[];
}

export interface UnorderedListToken extends Token {
	type: TokenType.UnorderedList;
	children: ListItemToken[];
}

export interface ListItemToken extends Token {
	type: TokenType.ListItem;
	children: Token[];
}

/**
 * Table tokens
 */
export interface TableToken extends Token {
	type: TokenType.Table;
	children: TableRowToken[];
}

export interface TableRowToken extends Token {
	type: TokenType.TableRow;
	isHeader: boolean;
	children: TableCellToken[];
}

export interface TableCellToken extends Token {
	type: TokenType.TableCell;
	align?: 'left' | 'center' | 'right';
	children: Token[];
}

/**
 * Horizontal rule token
 */
export interface HorizontalRuleToken extends Token {
	type: TokenType.HorizontalRule;
}

/**
 * LaTeX tokens
 */
export interface LatexBlockToken extends Token {
	type: TokenType.LatexBlock;
	content: string;
}

export interface LatexInlineToken extends Token {
	type: TokenType.LatexInline;
	content: string;
}

/**
 * Inline formatting tokens
 */
export interface BoldToken extends Token {
	type: TokenType.Bold;
	children: Token[];
}

export interface ItalicToken extends Token {
	type: TokenType.Italic;
	children: Token[];
}

export interface StrikethroughToken extends Token {
	type: TokenType.Strikethrough;
	children: Token[];
}

export interface InlineCodeToken extends Token {
	type: TokenType.InlineCode;
	content: string;
}

/**
 * Link and image tokens
 */
export interface LinkToken extends Token {
	type: TokenType.Link;
	url: string;
	title?: string;
	children: Token[];
}

export interface ImageToken extends Token {
	type: TokenType.Image;
	url: string;
	alt: string;
	title?: string;
}

/**
 * Citation token (custom)
 */
export interface CitationToken extends Token {
	type: TokenType.Citation;
	id: string;
}

/**
 * Line break token
 */
export interface LineBreakToken extends Token {
	type: TokenType.LineBreak;
}

/**
 * Parse result containing tokens and region boundaries
 */
export interface ParseResult {
	tokens: Token[];
	regions: RegionBoundary[];
}

/**
 * Region boundary for fake incremental parsing
 */
export interface RegionBoundary {
	type: 'blockquote' | 'table' | 'list' | 'codeblock' | 'paragraph';
	start: number;
	end: number;
}
