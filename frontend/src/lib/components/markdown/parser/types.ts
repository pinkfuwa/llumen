// --- Token constants (from streaming-markdown) ---
export const DOCUMENT = 1,
	PARAGRAPH = 2,
	HEADING_1 = 3,
	HEADING_2 = 4,
	HEADING_3 = 5,
	HEADING_4 = 6,
	HEADING_5 = 7,
	HEADING_6 = 8,
	CODE_BLOCK = 9,
	CODE_FENCE = 10,
	CODE_INLINE = 11,
	ITALIC_AST = 12,
	ITALIC_UND = 13,
	STRONG_AST = 14,
	STRONG_UND = 15,
	STRIKE = 16,
	LINK = 17,
	RAW_URL = 18,
	IMAGE = 19,
	BLOCKQUOTE = 20,
	LINE_BREAK = 21,
	RULE = 22,
	LIST_UNORDERED = 23,
	LIST_ORDERED = 24,
	LIST_ITEM = 25,
	CHECKBOX = 26,
	TABLE = 27,
	TABLE_ROW = 28,
	TABLE_CELL = 29,
	EQUATION_BLOCK = 30,
	EQUATION_INLINE = 31,
	EQUATION_BLOCK_DOLLAR = 32,
	EQUATION_BLOCK_BRACKET = 33,
	NEWLINE = 101,
	MAYBE_URL = 102,
	MAYBE_TASK = 103,
	MAYBE_BR = 104,
	MAYBE_EQ_BLOCK = 105,
	MAYBE_LINK = 106;

export type Token = number;

export const HREF = 1,
	SRC = 2,
	LANG = 4,
	CHECKED = 8,
	START = 16;

export type Attr = number;

export interface Parser {
	renderer: Renderer;
	text: string;
	pending: string;
	tokens: Uint32Array;
	len: number;
	token: number;
	spaces: Uint8Array;
	indent: string;
	indent_len: number;
	fence_end: number;
	fence_start: number;
	blockquote_idx: number;
	hr_char: string;
	hr_chars: number;
	table_state: number;
	eq_open: number;
	maybe_link_text: string;
}

export interface RendererData {
	nodes: unknown[];
	index: number;
}

export interface Renderer {
	data: RendererData;
	add_token: (data: RendererData, type: number) => void;
	end_token: (data: RendererData) => void;
	add_text: (data: RendererData, text: string) => void;
	set_attr: (data: RendererData, type: number, value: string) => void;
}

// --- AST node types (for Svelte rendering) ---
export enum AstNodeType {
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
	HorizontalRule = 'HorizontalRule',
	LatexBlock = 'LatexBlock',

	Text = 'Text',
	Bold = 'Bold',
	Italic = 'Italic',
	Strikethrough = 'Strikethrough',
	InlineCode = 'InlineCode',
	Link = 'Link',
	Image = 'Image',
	LatexInline = 'LatexInline',
	LineBreak = 'LineBreak'
}

export interface AstNode {
	type: AstNodeType;
	start: number;
	end: number;
	children?: AstNode[];
}

export interface TextNode extends AstNode {
	type: AstNodeType.Text;
	content: string;
}

export interface HeadingNode extends AstNode {
	type: AstNodeType.Heading;
	level: number;
	children: AstNode[];
}

export interface ParagraphNode extends AstNode {
	type: AstNodeType.Paragraph;
	children: AstNode[];
}

export interface CodeBlockNode extends AstNode {
	type: AstNodeType.CodeBlock;
	language?: string;
	content: string;
	closed: boolean;
}

export interface BlockquoteNode extends AstNode {
	type: AstNodeType.Blockquote;
	children: AstNode[];
}

export interface OrderedListNode extends AstNode {
	type: AstNodeType.OrderedList;
	startNumber?: number;
	children: ListItemNode[];
}

export interface UnorderedListNode extends AstNode {
	type: AstNodeType.UnorderedList;
	children: ListItemNode[];
}

export interface ListItemNode extends AstNode {
	type: AstNodeType.ListItem;
	children: AstNode[];
}

export interface TableNode extends AstNode {
	type: AstNodeType.Table;
	children: TableRowNode[];
}

export interface TableRowNode extends AstNode {
	type: AstNodeType.TableRow;
	isHeader: boolean;
	children: TableCellNode[];
}

export interface TableCellNode extends AstNode {
	type: AstNodeType.TableCell;
	align?: 'left' | 'center' | 'right';
	children: AstNode[];
}

export interface HorizontalRuleNode extends AstNode {
	type: AstNodeType.HorizontalRule;
}

export interface LatexBlockNode extends AstNode {
	type: AstNodeType.LatexBlock;
	content: string;
}

export interface LatexInlineNode extends AstNode {
	type: AstNodeType.LatexInline;
	content: string;
}

export interface BoldNode extends AstNode {
	type: AstNodeType.Bold;
	children: AstNode[];
}

export interface ItalicNode extends AstNode {
	type: AstNodeType.Italic;
	children: AstNode[];
}

export interface StrikethroughNode extends AstNode {
	type: AstNodeType.Strikethrough;
	children: AstNode[];
}

export interface InlineCodeNode extends AstNode {
	type: AstNodeType.InlineCode;
	content: string;
}

export interface LinkNode extends AstNode {
	type: AstNodeType.Link;
	url: string;
	title?: string;
	children: AstNode[];
}

export interface ImageNode extends AstNode {
	type: AstNodeType.Image;
	url: string;
	alt: string;
	title?: string;
}

export interface LineBreakNode extends AstNode {
	type: AstNodeType.LineBreak;
}

export interface RegionBoundary {
	type: 'blockquote' | 'table' | 'list' | 'codeblock' | 'paragraph';
	start: number;
	end: number;
}

export interface ParseResult {
	nodes: AstNode[];
	regions: RegionBoundary[];
}
