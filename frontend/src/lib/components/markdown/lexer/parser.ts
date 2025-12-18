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
	CitationToken,
	LineBreakToken,
	ParseResult,
	RegionBoundary
} from './tokens';
import { TokenType } from './tokens';

/**
 * Parser options for incremental-like parsing
 */
export interface ParserOptions {
	startFrom?: number;
}

/**
 * Main markdown parser class
 */
export class MarkdownParser {
	private source: string;
	private position: number;
	private regions: RegionBoundary[];

	constructor(source: string) {
		this.source = source;
		this.position = 0;
		this.regions = [];
	}

	/**
	 * Parse markdown content
	 */
	parse(options?: ParserOptions): ParseResult {
		this.position = options?.startFrom || 0;
		this.regions = [];

		const tokens: Token[] = [];

		while (this.position < this.source.length) {
			const token = this.parseBlock();
			if (token) {
				tokens.push(token);
			}
		}

		return {
			tokens,
			regions: this.regions
		};
	}

	/**
	 * Parse a block-level element
	 */
	private parseBlock(): Token | null {
		this.skipWhitespace();

		if (this.position >= this.source.length) {
			return null;
		}

		const start = this.position;

		// Try parsing different block types
		const heading = this.tryParseHeading();
		if (heading) return heading;

		const codeBlock = this.tryParseCodeBlock();
		if (codeBlock) {
			this.regions.push({
				type: 'codeblock',
				start: codeBlock.start,
				end: codeBlock.end
			});
			return codeBlock;
		}

		const latexBlock = this.tryParseLatexBlock();
		if (latexBlock) return latexBlock;

		const horizontalRule = this.tryParseHorizontalRule();
		if (horizontalRule) return horizontalRule;

		const citation = this.tryParseCitationBlock();
		if (citation) return citation;

		const table = this.tryParseTable();
		if (table) {
			this.regions.push({
				type: 'table',
				start: table.start,
				end: table.end
			});
			return table;
		}

		const blockquote = this.tryParseBlockquote();
		if (blockquote) {
			this.regions.push({
				type: 'blockquote',
				start: blockquote.start,
				end: blockquote.end
			});
			return blockquote;
		}

		const list = this.tryParseList();
		if (list) {
			this.regions.push({
				type: 'list',
				start: list.start,
				end: list.end
			});
			return list;
		}

		const paragraph = this.tryParseParagraph();
		if (paragraph) {
			// Track paragraphs as regions for incremental parsing
			this.regions.push({
				type: 'paragraph',
				start: paragraph.start,
				end: paragraph.end
			});
			return paragraph;
		}

		this.position++;
		return null;
	}

	/**
	 * Try to parse a heading (ATX style: # Heading)
	 */
	private tryParseHeading(): HeadingToken | null {
		const start = this.position;
		const line = this.peekLine();

		const match = line.match(/^(#{1,6})\s+(.+)$/);
		if (!match) {
			return null;
		}

		const level = match[1].length;
		const content = match[2];

		this.position += line.length;
		this.skipNewlines();

		const inlineTokens = this.parseInline(content, start + match[1].length + 1);

		return {
			type: TokenType.Heading,
			level,
			start,
			end: this.position,
			children: inlineTokens
		};
	}

	/**
	 * Try to parse a code block (fenced with ```)
	 * Supports open code blocks (without closing ```) for streaming
	 * Supports indentation up to 3 spaces (CommonMark spec)
	 */
	private tryParseCodeBlock(): CodeBlockToken | null {
		const start = this.position;
		const line = this.peekLine();

		const match = line.match(/^ {0,3}```(\w*)$/);
		if (!match) {
			return null;
		}

		const language = match[1] || undefined;
		this.position += line.length;
		this.skipNewlines();

		const contentStart = this.position;
		let contentEnd = this.position;

		while (this.position < this.source.length) {
			const currentLine = this.peekLine();
			// Also allow indented closing fence (up to 3 spaces)
			if (currentLine.match(/^ {0,3}```\s*$/)) {
				contentEnd = this.position;
				this.position += currentLine.length;
				this.skipNewlines();
				break;
			}
			this.position += currentLine.length;
			this.skipNewlines();
		}

		// If we reached EOF without finding closing delimiter, treat as open code block
		if (contentEnd === contentStart && this.position >= this.source.length) {
			contentEnd = this.position;
		}

		const content = this.source.substring(contentStart, contentEnd).trimEnd();

		return {
			type: TokenType.CodeBlock,
			language,
			content,
			start,
			end: this.position
		};
	}

	/**
	 * Try to parse a LaTeX block (\[ ... \] or $$ ... $$)
	 */
	private tryParseLatexBlock(): LatexBlockToken | null {
		const start = this.position;
		let delimiter: string;
		let endDelimiter: string;

		if (this.peek(2) === '\\[') {
			delimiter = '\\[';
			endDelimiter = '\\]';
		} else if (this.peek(2) === '$$') {
			delimiter = '$$';
			endDelimiter = '$$';
		} else {
			return null;
		}

		const afterDelimiter = this.peek(delimiter.length + 1);
		if (delimiter === '$$' && afterDelimiter[delimiter.length] !== '\n') {
			return null;
		}

		this.position += delimiter.length;
		if (delimiter === '$$') {
			this.skipNewlines();
		}

		const contentStart = this.position;
		const endPos = this.source.indexOf(endDelimiter, this.position);

		if (endPos === -1) {
			this.position = start;
			return null;
		}

		const content = this.source.substring(contentStart, endPos);
		this.position = endPos + endDelimiter.length;
		this.skipNewlines();

		return {
			type: TokenType.LatexBlock,
			content: content.trim(),
			start,
			end: this.position
		};
	}

	/**
	 * Try to parse a horizontal rule (---, ___, ***)
	 */
	private tryParseHorizontalRule(): HorizontalRuleToken | null {
		const start = this.position;
		const line = this.peekLine();

		if (line.match(/^---+$/) || line.match(/^\*\*\*+$/) || line.match(/^___+$/)) {
			this.position += line.length;
			this.skipNewlines();

			return {
				type: TokenType.HorizontalRule,
				start,
				end: this.position
			};
		}

		return null;
	}

	/**
	 * Try to parse a citation block (<citation>...</citation>)
	 */
	private tryParseCitationBlock(): CitationToken | null {
		const start = this.position;
		const remaining = this.source.substring(this.position);

		// Check if this starts with <citation>
		if (!remaining.startsWith('<citation>')) {
			return null;
		}

		// Find the closing </citation> tag
		const endTagPos = remaining.indexOf('</citation>');
		if (endTagPos === -1) {
			return null;
		}

		const citationContent = remaining.substring(10, endTagPos); // Skip '<citation>'
		this.position += endTagPos + 11; // Move past '</citation>'
		this.skipNewlines();

		// Parse the citation fields
		let title: string | undefined;
		let url: string | undefined;
		let favicon: string | undefined;
		let authoritative = false;

		// Extract title
		const titleMatch = citationContent.match(/<title>(.*?)<\/title>/s);
		if (titleMatch) {
			title = titleMatch[1].trim();
		}

		// Extract url
		const urlMatch = citationContent.match(/<url>(.*?)<\/url>/s);
		if (urlMatch) {
			url = urlMatch[1].trim();
		}

		// Extract favicon
		const faviconMatch = citationContent.match(/<favicon>(.*?)<\/favicon>/s);
		if (faviconMatch) {
			favicon = faviconMatch[1].trim();
		}

		// Check for authoritative (self-closing or paired tag)
		if (
			citationContent.includes('<authoritative/>') ||
			citationContent.includes('<authoritative />') ||
			citationContent.includes('<authoritative></authoritative>')
		) {
			authoritative = true;
		}

		return {
			type: TokenType.Citation,
			id: url || title || 'unknown',
			title,
			url,
			favicon,
			authoritative,
			start,
			end: this.position
		};
	}

	/**
	 * Try to parse a table
	 */
	private tryParseTable(): TableToken | null {
		const start = this.position;
		const firstLine = this.peekLine();

		if (!this.isTableRow(firstLine)) {
			return null;
		}

		const savedPosition = this.position;
		this.position += firstLine.length;
		this.skipNewlines();

		const secondLine = this.peekLine();
		if (!this.isTableSeparator(secondLine)) {
			this.position = savedPosition;
			return null;
		}

		this.position = savedPosition;

		const rows: TableRowToken[] = [];
		let lineNum = 0;

		while (this.position < this.source.length) {
			const line = this.peekLine();

			if (lineNum === 1 && this.isTableSeparator(line)) {
				this.position += line.length;
				this.skipNewlines();
				lineNum++;
				continue;
			}

			if (!this.isTableRow(line)) {
				break;
			}

			const rowStart = this.position;
			const cells = this.parseTableRow(line, rowStart);
			this.position += line.length;
			const hasBlankLine = this.skipNewlines();

			rows.push({
				type: TokenType.TableRow,
				isHeader: lineNum === 0,
				children: cells,
				start: rowStart,
				end: this.position
			});

			lineNum++;

			// If we crossed a blank line, stop parsing this table
			if (hasBlankLine) {
				break;
			}
		}

		if (rows.length < 2) {
			this.position = start;
			return null;
		}

		return {
			type: TokenType.Table,
			children: rows,
			start,
			end: this.position
		};
	}

	/**
	 * Check if a line is a table row (contains pipes or tabs as separators)
	 */
	private isTableRow(line: string): boolean {
		const trimmed = line.trim();
		return trimmed.includes('|') || trimmed.includes('\t');
	}

	/**
	 * Check if a line is a table separator (|---|---|)
	 */
	private isTableSeparator(line: string): boolean {
		const trimmed = line.trim();
		return /^[\|\s]*:?-+:?[\|\s:]*(:?-+:?[\|\s]*)*$/.test(trimmed);
	}

	/**
	 * Parse a table row into cells
	 */
	private parseTableRow(line: string, rowStart: number): TableCellToken[] {
		const cells: TableCellToken[] = [];
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
			const inlineTokens = this.parseInline(part, cellStart);
			offset += part.length + 1;

			cells.push({
				type: TokenType.TableCell,
				children: inlineTokens,
				start: cellStart,
				end: offset
			});
		}

		return cells;
	}

	/**
	 * Try to parse a blockquote (> text)
	 */
	private tryParseBlockquote(): BlockquoteToken | null {
		const start = this.position;
		const line = this.peekLine();

		if (!line.startsWith('>')) {
			return null;
		}

		const lines: string[] = [];
		while (this.position < this.source.length) {
			const currentLine = this.peekLine();
			if (!currentLine.startsWith('>')) {
				break;
			}

			lines.push(currentLine.substring(1).trim());
			this.position += currentLine.length;
			const hasBlankLine = this.skipNewlines();

			// If we crossed a blank line, stop parsing this blockquote
			if (hasBlankLine) {
				break;
			}
		}

		const content = lines.join('\n');
		const nestedParser = new MarkdownParser(content);
		const nestedResult = nestedParser.parse();

		return {
			type: TokenType.Blockquote,
			children: nestedResult.tokens,
			start,
			end: this.position
		};
	}

	/**
	 * Try to parse a list (ordered or unordered)
	 */
	private tryParseList(): OrderedListToken | UnorderedListToken | null {
		const start = this.position;
		const line = this.peekLine();

		const orderedMatch = line.match(/^(\d+)\.\s+/);
		const unorderedMatch = line.match(/^[-*+]\s+/);

		if (!orderedMatch && !unorderedMatch) {
			return null;
		}

		const isOrdered = !!orderedMatch;
		const startNumber = orderedMatch ? parseInt(orderedMatch[1]) : undefined;
		const items: ListItemToken[] = [];

		while (this.position < this.source.length) {
			const currentLine = this.peekLine();
			const itemMatch = isOrdered
				? currentLine.match(/^(\d+)\.\s+(.*)$/)
				: currentLine.match(/^[-*+]\s+(.*)$/);

			if (!itemMatch) {
				break;
			}

			const itemStart = this.position;
			const itemContent = isOrdered ? itemMatch[2] : itemMatch[1];

			this.position += currentLine.length;
			const hasBlankLine = this.skipNewlines();

			const inlineTokens = this.parseInline(
				itemContent,
				itemStart + (currentLine.length - itemContent.length)
			);

			items.push({
				type: TokenType.ListItem,
				children: inlineTokens,
				start: itemStart,
				end: this.position
			});

			// If we crossed a blank line, stop parsing this list
			if (hasBlankLine) {
				break;
			}
		}

		if (items.length === 0) {
			this.position = start;
			return null;
		}

		if (isOrdered) {
			return {
				type: TokenType.OrderedList,
				startNumber: startNumber,
				children: items,
				start: start,
				end: this.position
			};
		} else {
			return {
				type: TokenType.UnorderedList,
				children: items,
				start: start,
				end: this.position
			};
		}
	}

	/**
	 * Try to parse a paragraph
	 */
	private tryParseParagraph(): ParagraphToken | null {
		const start = this.position;
		const contentEnd = this.position;

		while (this.position < this.source.length) {
			const line = this.peekLine();

			if (line.trim() === '') {
				break;
			}

			if (this.looksLikeBlockStart(line)) {
				break;
			}

			this.position += line.length;

			const nextChar = this.source[this.position];
			if (nextChar === '\n' || nextChar === '\r') {
				this.position++;
				if (nextChar === '\r' && this.source[this.position] === '\n') {
					this.position++;
				}
			}
		}

		if (this.position === start) {
			return null;
		}

		// Use the original source content to preserve line breaks and spaces
		const content = this.source.substring(start, this.position).trimEnd();
		const inlineTokens = this.parseInline(content, start);

		return {
			type: TokenType.Paragraph,
			children: inlineTokens,
			start,
			end: this.position
		};
	}

	/**
	 * Check if a line looks like the start of a block element
	 */
	private looksLikeBlockStart(line: string): boolean {
		return (
			line.match(/^#{1,6}\s/) !== null ||
			line.match(/^ {0,3}```/) !== null ||
			line.match(/^(---+|\*\*\*+|___+)$/) !== null ||
			line.startsWith('>') ||
			line.match(/^\d+\.\s/) !== null ||
			line.match(/^[-*+]\s/) !== null ||
			line.match(/^\\\[/) !== null ||
			line.match(/^\$\$/) !== null ||
			this.isTableRow(line)
		);
	}

	/**
	 * Parse inline elements
	 */
	private parseInline(text: string, baseOffset: number): Token[] {
		const foundTokens: Token[] = [];
		let position = 0;

		// First pass: find all formatted tokens and their positions
		while (position < text.length) {
			const latexInline = this.tryParseInlineLatex(text, position, baseOffset);
			if (latexInline) {
				if (latexInline.content) {
					foundTokens.push(latexInline);
				}
				position = latexInline.end - baseOffset;
				continue;
			}

			const image = this.tryParseImage(text, position, baseOffset);
			if (image) {
				foundTokens.push(image);
				position = image.end - baseOffset;
				continue;
			}

			const link = this.tryParseLink(text, position, baseOffset);
			if (link) {
				foundTokens.push(link);
				position = link.end - baseOffset;
				continue;
			}

			const citation = this.tryParseCitation(text, position, baseOffset);
			if (citation) {
				foundTokens.push(citation);
				position = citation.end - baseOffset;
				continue;
			}

			const inlineCode = this.tryParseInlineCode(text, position, baseOffset);
			if (inlineCode) {
				foundTokens.push(inlineCode);
				position = inlineCode.end - baseOffset;
				continue;
			}

			const bold = this.tryParseBold(text, position, baseOffset);
			if (bold) {
				foundTokens.push(bold);
				position = bold.end - baseOffset;
				continue;
			}

			const italic = this.tryParseItalic(text, position, baseOffset);
			if (italic) {
				foundTokens.push(italic);
				position = italic.end - baseOffset;
				continue;
			}

			const strikethrough = this.tryParseStrikethrough(text, position, baseOffset);
			if (strikethrough) {
				foundTokens.push(strikethrough);
				position = strikethrough.end - baseOffset;
				continue;
			}

			const lineBreak = this.tryParseLineBreak(text, position, baseOffset);
			if (lineBreak) {
				foundTokens.push(lineBreak);
				position = lineBreak.end - baseOffset;
				continue;
			}

			position++;
		}

		// Second pass: build final token array with text tokens filling gaps
		const tokens: Token[] = [];
		let currentPos = 0;

		for (const token of foundTokens) {
			const tokenStart = token.start - baseOffset;

			// Add text before this token if there's a gap
			if (tokenStart > currentPos) {
				const textContent = text.substring(currentPos, tokenStart);
				tokens.push({
					type: TokenType.Text,
					content: textContent,
					start: baseOffset + currentPos,
					end: token.start
				} as TextToken);
			}

			tokens.push(token);
			currentPos = token.end - baseOffset;
		}

		// Add remaining text after last token
		if (currentPos < text.length) {
			const textContent = text.substring(currentPos);
			tokens.push({
				type: TokenType.Text,
				content: textContent,
				start: baseOffset + currentPos,
				end: baseOffset + text.length
			} as TextToken);
		}

		// If no tokens found at all, return text as single text token
		if (tokens.length === 0) {
			tokens.push({
				type: TokenType.Text,
				content: text,
				start: baseOffset,
				end: baseOffset + text.length
			} as TextToken);
		}

		return this.mergeTextTokens(tokens);
	}

	/**
	 * Extract text content not covered by other tokens
	 */
	private extractText(text: string, tokens: Token[], baseOffset: number): string {
		if (tokens.length === 0) {
			return text;
		}

		let result = '';
		let lastEnd = 0;

		for (const token of tokens) {
			const tokenStart = token.start - baseOffset;
			if (tokenStart > lastEnd) {
				result += text.substring(lastEnd, tokenStart);
			}
			lastEnd = token.end - baseOffset;
		}

		if (lastEnd < text.length) {
			result += text.substring(lastEnd);
		}

		return result;
	}

	/**
	 * Merge consecutive text tokens
	 */
	private mergeTextTokens(tokens: Token[]): Token[] {
		const merged: Token[] = [];
		let currentText: TextToken | null = null;

		for (const token of tokens) {
			if (token.type === TokenType.Text) {
				if (currentText) {
					currentText.content += (token as TextToken).content;
					currentText.end = token.end;
				} else {
					currentText = { ...token } as TextToken;
				}
			} else {
				if (currentText) {
					merged.push(currentText);
					currentText = null;
				}
				merged.push(token);
			}
		}

		if (currentText) {
			merged.push(currentText);
		}

		return merged;
	}

	/**
	 * Try to parse inline LaTeX
	 */
	private tryParseInlineLatex(
		text: string,
		position: number,
		baseOffset: number
	): LatexInlineToken | null {
		const remaining = text.substring(position);

		if (remaining.startsWith('\\(')) {
			const endPos = remaining.indexOf('\\)', 2);
			if (endPos !== -1) {
				const content = remaining.substring(2, endPos);
				return {
					type: TokenType.LatexInline,
					content,
					start: baseOffset + position,
					end: baseOffset + position + endPos + 2
				};
			}
		}

		if (remaining.startsWith('$')) {
			const afterDollar = remaining.substring(1);

			if (afterDollar.startsWith('$')) {
				return null;
			}

			const endPos = afterDollar.indexOf('$');
			if (endPos === -1) {
				return null;
			}

			const content = afterDollar.substring(0, endPos);

			if (content.length === 0) {
				return null;
			}

			const startsWithSpace = content[0] === ' ';
			const endsWithSpace = content[content.length - 1] === ' ';

			// Reject asymmetric spacing (one side has space, other doesn't)
			if (startsWithSpace !== endsWithSpace) {
				return null;
			}

			const hasSpaces = /\s/.test(content);

			if (hasSpaces) {
				if (position > 0 && text[position - 1] !== ' ') {
					return null;
				}

				const closingPos = position + 1 + endPos + 1;
				if (closingPos < text.length && text[closingPos] !== ' ') {
					return null;
				}
			}

			return {
				type: TokenType.LatexInline,
				content,
				start: baseOffset + position,
				end: baseOffset + position + endPos + 2
			};
		}

		return null;
	}

	/**
	 * Try to parse an image
	 */
	private tryParseImage(text: string, position: number, baseOffset: number): ImageToken | null {
		const remaining = text.substring(position);
		const match = remaining.match(/^!\[([^\]]*)\]\(([^)]+)\)/);

		if (!match) {
			return null;
		}

		const alt = match[1];
		const url = match[2];

		return {
			type: TokenType.Image,
			alt,
			url,
			start: baseOffset + position,
			end: baseOffset + position + match[0].length
		};
	}

	/**
	 * Try to parse a link
	 */
	private tryParseLink(text: string, position: number, baseOffset: number): LinkToken | null {
		const remaining = text.substring(position);
		const match = remaining.match(/^\[([^\]]+)\]\(([^)]+)\)/);

		if (!match) {
			return null;
		}

		const linkText = match[1];
		const url = match[2];

		const inlineTokens = this.parseInline(linkText, baseOffset + position + 1);

		return {
			type: TokenType.Link,
			url,
			children: inlineTokens,
			start: baseOffset + position,
			end: baseOffset + position + match[0].length
		};
	}

	/**
	 * Try to parse a citation (custom: [@cite])
	 */
	private tryParseCitation(
		text: string,
		position: number,
		baseOffset: number
	): CitationToken | null {
		const remaining = text.substring(position);
		const match = remaining.match(/^\[@([^\]]+)\]/);

		if (!match) {
			return null;
		}

		return {
			type: TokenType.Citation,
			id: match[1],
			start: baseOffset + position,
			end: baseOffset + position + match[0].length,
			title: undefined,
			url: undefined,
			favicon: undefined,
			authoritative: false
		};
	}

	/**
	 * Try to parse inline code
	 */
	private tryParseInlineCode(
		text: string,
		position: number,
		baseOffset: number
	): InlineCodeToken | null {
		const remaining = text.substring(position);

		if (!remaining.startsWith('`')) {
			return null;
		}

		const endPos = remaining.indexOf('`', 1);
		if (endPos === -1) {
			return null;
		}

		const content = remaining.substring(1, endPos);

		return {
			type: TokenType.InlineCode,
			content,
			start: baseOffset + position,
			end: baseOffset + position + endPos + 1
		};
	}

	/**
	 * Try to parse bold text (** or __)
	 */
	private tryParseBold(text: string, position: number, baseOffset: number): BoldToken | null {
		const remaining = text.substring(position);

		if (remaining.startsWith('**')) {
			const endPos = remaining.indexOf('**', 2);
			if (endPos !== -1) {
				const content = remaining.substring(2, endPos);
				const inlineTokens = this.parseInline(content, baseOffset + position + 2);

				return {
					type: TokenType.Bold,
					children: inlineTokens,
					start: baseOffset + position,
					end: baseOffset + position + endPos + 2
				};
			}
		}

		if (remaining.startsWith('__')) {
			const endPos = remaining.indexOf('__', 2);
			if (endPos !== -1) {
				const content = remaining.substring(2, endPos);
				const inlineTokens = this.parseInline(content, baseOffset + position + 2);

				return {
					type: TokenType.Bold,
					children: inlineTokens,
					start: baseOffset + position,
					end: baseOffset + position + endPos + 2
				};
			}
		}

		return null;
	}

	/**
	 * Try to parse italic text (* or _)
	 */
	private tryParseItalic(text: string, position: number, baseOffset: number): ItalicToken | null {
		const remaining = text.substring(position);

		if (remaining.startsWith('*') && !remaining.startsWith('**')) {
			const endPos = remaining.indexOf('*', 1);
			if (endPos !== -1 && remaining[endPos + 1] !== '*') {
				const content = remaining.substring(1, endPos);
				const inlineTokens = this.parseInline(content, baseOffset + position + 1);

				return {
					type: TokenType.Italic,
					children: inlineTokens,
					start: baseOffset + position,
					end: baseOffset + position + endPos + 1
				};
			}
		}

		if (remaining.startsWith('_') && !remaining.startsWith('__')) {
			const endPos = remaining.indexOf('_', 1);
			if (endPos !== -1 && remaining[endPos + 1] !== '_') {
				const content = remaining.substring(1, endPos);
				const inlineTokens = this.parseInline(content, baseOffset + position + 1);

				return {
					type: TokenType.Italic,
					children: inlineTokens,
					start: baseOffset + position,
					end: baseOffset + position + endPos + 1
				};
			}
		}

		return null;
	}

	/**
	 * Try to parse strikethrough text (~~)
	 */
	private tryParseStrikethrough(
		text: string,
		position: number,
		baseOffset: number
	): StrikethroughToken | null {
		const remaining = text.substring(position);

		if (remaining.startsWith('~~')) {
			const endPos = remaining.indexOf('~~', 2);
			if (endPos !== -1) {
				const content = remaining.substring(2, endPos);
				const inlineTokens = this.parseInline(content, baseOffset + position + 2);

				return {
					type: TokenType.Strikethrough,
					children: inlineTokens,
					start: baseOffset + position,
					end: baseOffset + position + endPos + 2
				};
			}
		}

		return null;
	}

	/**
	 * Try to parse a line break (two spaces followed by newline)
	 */
	private tryParseLineBreak(
		text: string,
		position: number,
		baseOffset: number
	): LineBreakToken | null {
		const remaining = text.substring(position);

		// Check for two or more spaces followed by newline
		const match = remaining.match(/^( {2,})(\r?\n)/);
		if (match) {
			return {
				type: TokenType.LineBreak,
				start: baseOffset + position,
				end: baseOffset + position + match[0].length
			};
		}

		return null;
	}

	/**
	 * Peek the current line without consuming it
	 */
	private peekLine(): string {
		const newlinePos = this.source.indexOf('\n', this.position);
		if (newlinePos === -1) {
			return this.source.substring(this.position);
		}
		return this.source.substring(this.position, newlinePos);
	}

	/**
	 * Peek ahead n characters
	 */
	private peek(n: number): string {
		return this.source.substring(this.position, this.position + n);
	}

	/**
	 * Skip whitespace (spaces and tabs only, not newlines)
	 */
	private skipWhitespace(): void {
		while (
			this.position < this.source.length &&
			(this.source[this.position] === ' ' || this.source[this.position] === '\t')
		) {
			this.position++;
		}
	}

	/**
	 * Skip newlines and return true if we crossed a blank line
	 */
	private skipNewlines(): boolean {
		let newlineCount = 0;
		while (
			this.position < this.source.length &&
			(this.source[this.position] === '\n' || this.source[this.position] === '\r')
		) {
			if (this.source[this.position] === '\n') {
				newlineCount++;
			}
			this.position++;
		}
		// If we saw 2+ newlines, there was a blank line
		return newlineCount >= 2;
	}
}

/**
 * Parse markdown content
 */
export function parse(source: string, options?: ParserOptions): ParseResult {
	const parser = new MarkdownParser(source);
	return parser.parse(options);
}
