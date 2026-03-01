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
