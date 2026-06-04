import {
	DOCUMENT,
	PARAGRAPH,
	HEADING_1,
	HEADING_2,
	HEADING_3,
	HEADING_4,
	HEADING_5,
	HEADING_6,
	CODE_BLOCK,
	CODE_FENCE,
	CODE_INLINE,
	ITALIC_AST,
	ITALIC_UND,
	STRONG_AST,
	STRONG_UND,
	STRIKE,
	LINK,
	RAW_URL,
	IMAGE,
	BLOCKQUOTE,
	LINE_BREAK,
	RULE,
	LIST_UNORDERED,
	LIST_ORDERED,
	LIST_ITEM,
	CHECKBOX,
	TABLE,
	TABLE_ROW,
	TABLE_CELL,
	EQUATION_BLOCK,
	EQUATION_INLINE,
	HREF,
	SRC,
	LANG,
	START,
	CHECKED,
	type Renderer,
	type RendererData
} from './types';
import {
	AstNodeType,
	type AstNode,
	type HeadingNode,
	type CodeBlockNode,
	type TextNode
} from './types';
import { parser, parser_write, parser_end, heading_to_level } from './smd';

interface StackEntry {
	type: AstNodeType;
	token: number;
	children: AstNode[];
	textBuf: string;
	language?: string;
	url?: string;
	start?: number;
	checked?: boolean;
	rowCount: number;
	id: number;
}

let nodeIdCounter = 0;

function flushText(entry: StackEntry): void {
	if (entry.textBuf.length === 0) return;
	const id = ++nodeIdCounter;
	entry.children.push({
		type: AstNodeType.Text,
		start: id,
		end: id,
		content: entry.textBuf
	} as TextNode);
	entry.textBuf = '';
}

function tokenToNodeType(token: number): AstNodeType {
	switch (token) {
		case PARAGRAPH:
			return AstNodeType.Paragraph;
		case HEADING_1:
		case HEADING_2:
		case HEADING_3:
		case HEADING_4:
		case HEADING_5:
		case HEADING_6:
			return AstNodeType.Heading;
		case CODE_BLOCK:
		case CODE_FENCE:
			return AstNodeType.CodeBlock;
		case CODE_INLINE:
			return AstNodeType.InlineCode;
		case ITALIC_AST:
		case ITALIC_UND:
			return AstNodeType.Italic;
		case STRONG_AST:
		case STRONG_UND:
			return AstNodeType.Bold;
		case STRIKE:
			return AstNodeType.Strikethrough;
		case LINK:
		case RAW_URL:
			return AstNodeType.Link;
		case IMAGE:
			return AstNodeType.Image;
		case BLOCKQUOTE:
			return AstNodeType.Blockquote;
		case LINE_BREAK:
			return AstNodeType.LineBreak;
		case RULE:
			return AstNodeType.HorizontalRule;
		case LIST_UNORDERED:
			return AstNodeType.UnorderedList;
		case LIST_ORDERED:
			return AstNodeType.OrderedList;
		case LIST_ITEM:
			return AstNodeType.ListItem;
		case TABLE:
			return AstNodeType.Table;
		case TABLE_ROW:
			return AstNodeType.TableRow;
		case TABLE_CELL:
			return AstNodeType.TableCell;
		case EQUATION_BLOCK:
			return AstNodeType.LatexBlock;
		case EQUATION_INLINE:
			return AstNodeType.LatexInline;
		case CHECKBOX:
			return AstNodeType.Text;
		default:
			return AstNodeType.Paragraph;
	}
}

function finalizeNode(entry: StackEntry): AstNode {
	const base = { start: entry.id, end: entry.id };

	switch (entry.type) {
		case AstNodeType.Heading: {
			const level = heading_to_level(entry.token);
			return { ...base, type: AstNodeType.Heading, level, children: entry.children } as HeadingNode;
		}
		case AstNodeType.CodeBlock: {
			let content = '';
			for (const child of entry.children) {
				if (child.type === AstNodeType.Text) content += (child as TextNode).content;
			}
			return {
				...base,
				type: AstNodeType.CodeBlock,
				language: entry.language,
				content,
				closed: true
			} as CodeBlockNode;
		}
		case AstNodeType.InlineCode: {
			let content = '';
			for (const child of entry.children) {
				if (child.type === AstNodeType.Text) content += (child as TextNode).content;
			}
			return { ...base, type: AstNodeType.InlineCode, content } as AstNode;
		}
		case AstNodeType.Image: {
			let alt = '';
			for (const child of entry.children) {
				if (child.type === AstNodeType.Text) alt += (child as TextNode).content;
			}
			return { ...base, type: AstNodeType.Image, url: entry.url || '', alt } as AstNode;
		}
		case AstNodeType.Link: {
			return {
				...base,
				type: AstNodeType.Link,
				url: entry.url || '#',
				children: entry.children
			} as AstNode;
		}
		case AstNodeType.LatexBlock: {
			let content = '';
			for (const child of entry.children) {
				if (child.type === AstNodeType.Text) content += (child as TextNode).content;
			}
			return { ...base, type: AstNodeType.LatexBlock, content } as AstNode;
		}
		case AstNodeType.LatexInline: {
			let content = '';
			for (const child of entry.children) {
				if (child.type === AstNodeType.Text) content += (child as TextNode).content;
			}
			return { ...base, type: AstNodeType.LatexInline, content } as AstNode;
		}
		case AstNodeType.TableRow: {
			const isHeader = entry.rowCount === 0;
			return { ...base, type: AstNodeType.TableRow, isHeader, children: entry.children } as AstNode;
		}
		case AstNodeType.OrderedList: {
			return {
				...base,
				type: AstNodeType.OrderedList,
				startNumber: entry.start,
				children: entry.children
			} as AstNode;
		}
		case AstNodeType.Text: {
			const text = entry.checked != null ? (entry.checked ? '[x] ' : '[ ] ') : '';
			return { ...base, type: AstNodeType.Text, content: text } as AstNode;
		}
		default:
			return { ...base, type: entry.type, children: entry.children } as AstNode;
	}
}

export function createAstRenderer(): { renderer: Renderer; getResult: () => AstNode[] } {
	const stack: StackEntry[] = [
		{ type: AstNodeType.Paragraph, token: DOCUMENT, children: [], textBuf: '', rowCount: 0, id: 0 }
	];

	const renderer: Renderer = {
		data: { nodes: [], index: 0 },
		add_token(_data: RendererData, token: number): void {
			if (token === DOCUMENT) return;

			if (token === LINE_BREAK) {
				flushText(stack[stack.length - 1]);
			}

			if (token === TABLE_ROW) {
				const tableEntry = getTableEntry();
				if (tableEntry) tableEntry.rowCount += 1;
			}

			const nodeType = tokenToNodeType(token);
			const entry: StackEntry = {
				type: nodeType,
				token,
				children: [],
				textBuf: '',
				rowCount: 0,
				id: ++nodeIdCounter
			};
			stack.push(entry);
		},
		end_token(_data: RendererData): void {
			if (stack.length <= 1) return;
			const entry = stack.pop()!;
			flushText(entry);

			const node = finalizeNode(entry);
			stack[stack.length - 1].children.push(node);
		},
		add_text(_data: RendererData, text: string): void {
			const current = stack[stack.length - 1];
			if (current.token === DOCUMENT) return;
			if (
				current.type === AstNodeType.LineBreak ||
				current.type === AstNodeType.HorizontalRule ||
				current.type === AstNodeType.Text
			)
				return;
			current.textBuf += text;
		},
		set_attr(_data: RendererData, type: number, value: string): void {
			const current = stack[stack.length - 1];
			switch (type) {
				case HREF:
					current.url = value;
					break;
				case SRC:
					current.url = value;
					break;
				case LANG:
					current.language = value;
					break;
				case START:
					current.start = parseInt(value, 10) || undefined;
					break;
				case CHECKED:
					current.checked = true;
					break;
			}
		}
	};

	function getTableEntry(): StackEntry | undefined {
		for (let i = stack.length - 1; i >= 0; i--) {
			if (stack[i].type === AstNodeType.Table) return stack[i];
		}
		return undefined;
	}

	function getResult(): AstNode[] {
		while (stack.length > 1) {
			const entry = stack.pop()!;
			flushText(entry);
			stack[stack.length - 1].children.push(finalizeNode(entry));
		}
		return stack[0].children;
	}

	return { renderer, getResult };
}

export function parseSync(source: string): AstNode[] {
	const { renderer, getResult } = createAstRenderer();
	const p = parser(renderer);
	parser_write(p, source);
	parser_end(p);
	return getResult();
}
