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
	EQUATION_BLOCK_DOLLAR,
	EQUATION_BLOCK_BRACKET,
	EQUATION_INLINE,
	FOOTNOTE_REF,
	FOOTNOTE_DEF,
	HREF,
	SRC,
	LANG,
	START,
	CHECKED,
	LABEL,
	type Renderer,
	type RendererData
} from './types';
import {
	AstNodeType,
	type AstNode,
	type HeadingNode,
	type CodeBlockNode,
	type TextNode,
	type FootnoteRefNode,
	type FootnoteDefNode
} from './types';
import { parser, parser_write, parser_end, heading_to_level } from './smd';

interface StackEntry {
	token: number;
	node: AstNode;
	textBuf: string;
	textStart: number;
	textEnd: number;
	language?: string;
	url?: string;
	start?: number;
	checked?: boolean;
	label?: string;
	rowCount: number;
	closed: boolean;
}

function flushText(entry: StackEntry): void {
	if (entry.textBuf.length === 0) return;
	const children = entry.node.children!;
	const last = children.length > 0 ? children[children.length - 1] : null;
	if (last?.type === AstNodeType.Text) {
		(last as TextNode).content += entry.textBuf;
		(last as TextNode).end = entry.textEnd;
	} else {
		children.push({
			type: AstNodeType.Text,
			start: entry.textStart,
			end: entry.textEnd,
			content: entry.textBuf
		} as TextNode);
	}
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
		case EQUATION_BLOCK_DOLLAR:
		case EQUATION_BLOCK_BRACKET:
			return AstNodeType.LatexBlock;
		case EQUATION_INLINE:
			return AstNodeType.LatexInline;
		case FOOTNOTE_REF:
			return AstNodeType.FootnoteRef;
		case FOOTNOTE_DEF:
			return AstNodeType.FootnoteDef;
		case CHECKBOX:
			return AstNodeType.Text;
		default:
			return AstNodeType.Paragraph;
	}
}

function finalizeNodeInPlace(entry: StackEntry): void {
	const node = entry.node;
	switch (node.type) {
		case AstNodeType.Heading: {
			(node as HeadingNode).level = heading_to_level(entry.token);
			break;
		}
		case AstNodeType.CodeBlock: {
			let content = '';
			for (const child of node.children!) {
				if (child.type === AstNodeType.Text) content += (child as TextNode).content;
			}
			(node as CodeBlockNode).content = content;
			(node as CodeBlockNode).closed = entry.closed;
			(node as CodeBlockNode).language = entry.language;
			break;
		}
		case AstNodeType.InlineCode: {
			let content = '';
			for (const child of node.children!) {
				if (child.type === AstNodeType.Text) content += (child as TextNode).content;
			}
			(node as any).content = content;
			break;
		}
		case AstNodeType.Image: {
			let alt = '';
			for (const child of node.children!) {
				if (child.type === AstNodeType.Text) alt += (child as TextNode).content;
			}
			(node as any).url = entry.url || '';
			(node as any).alt = alt;
			break;
		}
		case AstNodeType.Link: {
			(node as any).url = entry.url || '#';
			break;
		}
		case AstNodeType.FootnoteRef: {
			(node as FootnoteRefNode).label = entry.label || '';
			break;
		}
		case AstNodeType.FootnoteDef: {
			(node as FootnoteDefNode).label = entry.label || '';
			break;
		}
		case AstNodeType.LatexBlock:
		case AstNodeType.LatexInline: {
			let content = '';
			for (const child of node.children!) {
				if (child.type === AstNodeType.Text) content += (child as TextNode).content;
			}
			(node as any).content = content;
			break;
		}
		case AstNodeType.TableRow: {
			(node as any).isHeader = entry.rowCount === 0;
			break;
		}
		case AstNodeType.OrderedList: {
			(node as any).startNumber = entry.start;
			break;
		}
		case AstNodeType.Text: {
			const text = entry.checked != null ? (entry.checked ? '[x] ' : '[ ] ') : '';
			(node as TextNode).content = text;
			break;
		}
	}
}

function getTableEntry(stack: StackEntry[]): StackEntry | undefined {
	for (let i = stack.length - 1; i >= 0; i--) {
		if (stack[i].node.type === AstNodeType.Table) return stack[i];
	}
	return undefined;
}

export function createAstRenderer(rootChildren: AstNode[] = []): {
	renderer: Renderer;
	getResult: () => AstNode[];
} {
	const stack: StackEntry[] = [
		{
			token: DOCUMENT,
			node: { type: AstNodeType.Paragraph, start: 0, end: 0, children: rootChildren },
			textBuf: '',
			textStart: 0,
			textEnd: 0,
			rowCount: 0,
			closed: true
		}
	];

	const renderer: Renderer = {
		data: { nodes: [], index: 0, pos: 0, pendingLen: 0, textStart: 0 },
		add_token(data: RendererData, token: number): void {
			if (token === DOCUMENT) return;

			const parent = stack[stack.length - 1];
			flushText(parent);

			if (token === TABLE_ROW) {
				const tableEntry = getTableEntry(stack);
				if (tableEntry) tableEntry.rowCount += 1;
			}

			const nodeType = tokenToNodeType(token);
			const children: AstNode[] = [];
			const node: AstNode =
				nodeType === AstNodeType.CodeBlock
					? ({
							type: nodeType,
							start: data.pos,
							end: 0,
							children,
							content: '',
							closed: false
						} as unknown as AstNode)
					: { type: nodeType, start: data.pos, end: 0, children };
			parent.node.children!.push(node);
			const proxiedNode = parent.node.children![parent.node.children!.length - 1] as AstNode;

			stack.push({
				token,
				node: proxiedNode,
				textBuf: '',
				textStart: 0,
				textEnd: 0,
				language: undefined,
				url: undefined,
				start: undefined,
				checked: undefined,
				rowCount: 0,
				closed: false
			});
		},
		end_token(data: RendererData): void {
			if (stack.length <= 1) return;
			const entry = stack.pop()!;
			flushText(entry);
			entry.node.end = data.pos;
			entry.closed = true;
			finalizeNodeInPlace(entry);
		},
		add_text(data: RendererData, text: string): void {
			const current = stack[stack.length - 1];
			if (current.token === DOCUMENT) return;
			if (
				current.node.type === AstNodeType.LineBreak ||
				current.node.type === AstNodeType.HorizontalRule ||
				current.node.type === AstNodeType.Text
			)
				return;
			if (current.textBuf.length === 0) {
				current.textStart = data.textStart;
			}
			current.textBuf += text;
			current.textEnd = data.textStart + text.length;
			if (current.node.type === AstNodeType.CodeBlock) {
				(current.node as CodeBlockNode).content += text;
			}
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
					if (current.node.type === AstNodeType.CodeBlock) {
						(current.node as CodeBlockNode).language = value;
					}
					break;
				case START:
					current.start = parseInt(value, 10) || undefined;
					break;
				case CHECKED:
					current.checked = true;
					break;
				case LABEL:
					current.label = value;
					break;
			}
		}
	};

	function getResult(): AstNode[] {
		while (stack.length > 1) {
			const entry = stack.pop()!;
			flushText(entry);
			entry.closed = true;
			finalizeNodeInPlace(entry);
		}
		return rootChildren;
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
