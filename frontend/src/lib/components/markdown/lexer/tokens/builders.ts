import { TokenType } from '../tokens';
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
	LineBreakToken
} from '../tokens';

export function createTextToken(content: string, start: number, end: number): TextToken {
	return {
		type: TokenType.Text,
		content,
		start,
		end
	};
}

export function createHeadingToken(
	level: number,
	children: Token[],
	start: number,
	end: number
): HeadingToken {
	return {
		type: TokenType.Heading,
		level,
		children,
		start,
		end
	};
}

export function createParagraphToken(
	children: Token[],
	start: number,
	end: number
): ParagraphToken {
	return {
		type: TokenType.Paragraph,
		children,
		start,
		end
	};
}

export function createCodeBlockToken(
	language: string | undefined,
	content: string,
	closed: boolean,
	start: number,
	end: number
): CodeBlockToken {
	return {
		type: TokenType.CodeBlock,
		language,
		content,
		closed,
		start,
		end
	};
}

export function createBlockquoteToken(
	children: Token[],
	start: number,
	end: number
): BlockquoteToken {
	return {
		type: TokenType.Blockquote,
		children,
		start,
		end
	};
}

export function createOrderedListToken(
	startNumber: number | undefined,
	children: Token[],
	start: number,
	end: number
): OrderedListToken {
	return {
		type: TokenType.OrderedList,
		startNumber,
		children: children as OrderedListToken['children'],
		start,
		end
	};
}

export function createUnorderedListToken(
	children: Token[],
	start: number,
	end: number
): UnorderedListToken {
	return {
		type: TokenType.UnorderedList,
		children: children as UnorderedListToken['children'],
		start,
		end
	};
}

export function createListItemToken(children: Token[], start: number, end: number): ListItemToken {
	return {
		type: TokenType.ListItem,
		children: children as ListItemToken['children'],
		start,
		end
	};
}

export function createTableToken(children: Token[], start: number, end: number): TableToken {
	return {
		type: TokenType.Table,
		children: children as TableToken['children'],
		start,
		end
	};
}

export function createTableRowToken(
	isHeader: boolean,
	children: Token[],
	start: number,
	end: number
): TableRowToken {
	return {
		type: TokenType.TableRow,
		isHeader,
		children: children as TableRowToken['children'],
		start,
		end
	};
}

export function createTableCellToken(
	align: 'left' | 'center' | 'right' | undefined,
	children: Token[],
	start: number,
	end: number
): TableCellToken {
	return {
		type: TokenType.TableCell,
		align,
		children: children as TableCellToken['children'],
		start,
		end
	};
}

export function createHorizontalRuleToken(start: number, end: number): HorizontalRuleToken {
	return {
		type: TokenType.HorizontalRule,
		start,
		end
	};
}

export function createLatexBlockToken(
	content: string,
	start: number,
	end: number
): LatexBlockToken {
	return {
		type: TokenType.LatexBlock,
		content,
		start,
		end
	};
}

export function createLatexInlineToken(
	content: string,
	start: number,
	end: number
): LatexInlineToken {
	return {
		type: TokenType.LatexInline,
		content,
		start,
		end
	};
}

export function createBoldToken(children: Token[], start: number, end: number): BoldToken {
	return {
		type: TokenType.Bold,
		children,
		start,
		end
	};
}

export function createItalicToken(children: Token[], start: number, end: number): ItalicToken {
	return {
		type: TokenType.Italic,
		children,
		start,
		end
	};
}

export function createStrikethroughToken(
	children: Token[],
	start: number,
	end: number
): StrikethroughToken {
	return {
		type: TokenType.Strikethrough,
		children,
		start,
		end
	};
}

export function createInlineCodeToken(
	content: string,
	start: number,
	end: number
): InlineCodeToken {
	return {
		type: TokenType.InlineCode,
		content,
		start,
		end
	};
}

export function createLinkToken(
	url: string,
	children: Token[],
	start: number,
	end: number
): LinkToken {
	return {
		type: TokenType.Link,
		url,
		children,
		start,
		end
	};
}

export function createImageToken(url: string, alt: string, start: number, end: number): ImageToken {
	return {
		type: TokenType.Image,
		url,
		alt,
		start,
		end
	};
}

export function createLineBreakToken(start: number, end: number): LineBreakToken {
	return {
		type: TokenType.LineBreak,
		start,
		end
	};
}
