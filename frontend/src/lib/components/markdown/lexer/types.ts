export enum LexTokenKind {
	// Block markers
	HeadingMarker = 'HeadingMarker',
	FenceOpen = 'FenceOpen',
	FenceClose = 'FenceClose',
	HorizontalRule = 'HorizontalRule',
	BlockquoteMarker = 'BlockquoteMarker',
	ListItemMarker = 'ListItemMarker',
	TablePipe = 'TablePipe',
	TableSeparatorRow = 'TableSeparatorRow',

	// Inline delimiters
	BoldDelim = 'BoldDelim',
	ItalicDelim = 'ItalicDelim',
	StrikethroughDelim = 'StrikethroughDelim',
	CodeSpanDelim = 'CodeSpanDelim',
	LatexInlineOpen = 'LatexInlineOpen',
	LatexInlineClose = 'LatexInlineClose',
	LatexBlockOpen = 'LatexBlockOpen',
	LatexBlockClose = 'LatexBlockClose',
	LinkStart = 'LinkStart',
	LinkEnd = 'LinkEnd',
	ImageStart = 'ImageStart',
	LineBreak = 'LineBreak',

	// Content
	Text = 'Text',
	Newline = 'Newline',
	BlankLine = 'BlankLine'
}

export interface LexToken {
	kind: LexTokenKind;
	start: number;
	end: number;
	/** Semantic payload — meaning depends on kind:
	 *  - HeadingMarker: level as string ("1"-"6")
	 *  - FenceOpen: language identifier
	 *  - Text: the text content
	 *  - ListItemMarker: "- ", "* ", "1. " etc (the raw marker)
	 *  - LinkEnd: url
	 *  - ImageStart: "alt|url" (pipe-separated)
	 */
	value: string;
}
