import { LexTokenKind, type LexToken } from '../lexer/types';
import {
	AstNodeType,
	type AstNode,
	type TextNode,
	type BoldNode,
	type ItalicNode,
	type StrikethroughNode,
	type InlineCodeNode,
	type LinkNode,
	type ImageNode,
	type LatexInlineNode,
	type LineBreakNode
} from './types';
import { lexInline } from '../lexer/inline';

/**
 * Build inline AST from lex tokens.
 * This is called on Text spans extracted by the block parser.
 */
export function buildInlineAst(text: string, baseOffset: number): AstNode[] {
	const lexTokens = lexInline(text, baseOffset);
	return buildInlineFromLexTokens(lexTokens);
}

/**
 * Convert flat lex tokens into a nested AST.
 * Delimiters (Bold, Italic, etc.) are matched as open/close pairs,
 * and content between them becomes children.
 */
export function buildInlineFromLexTokens(tokens: LexToken[]): AstNode[] {
	const result: AstNode[] = [];
	let i = 0;

	while (i < tokens.length) {
		const token = tokens[i];

		switch (token.kind) {
			case LexTokenKind.BoldDelim: {
				const closeIdx = findClosingDelim(tokens, i + 1, LexTokenKind.BoldDelim);
				if (closeIdx !== -1) {
					const innerTokens = tokens.slice(i + 1, closeIdx);
					const children = buildInlineFromLexTokens(innerTokens);
					const node: BoldNode = {
						type: AstNodeType.Bold,
						start: token.start,
						end: tokens[closeIdx].end,
						children
					};
					result.push(node);
					i = closeIdx + 1;
				} else {
					pushText(result, token);
					i++;
				}
				break;
			}

			case LexTokenKind.ItalicDelim: {
				const closeIdx = findClosingDelim(tokens, i + 1, LexTokenKind.ItalicDelim);
				if (closeIdx !== -1) {
					const innerTokens = tokens.slice(i + 1, closeIdx);
					const children = buildInlineFromLexTokens(innerTokens);
					const node: ItalicNode = {
						type: AstNodeType.Italic,
						start: token.start,
						end: tokens[closeIdx].end,
						children
					};
					result.push(node);
					i = closeIdx + 1;
				} else {
					pushText(result, token);
					i++;
				}
				break;
			}

			case LexTokenKind.StrikethroughDelim: {
				const closeIdx = findClosingDelim(tokens, i + 1, LexTokenKind.StrikethroughDelim);
				if (closeIdx !== -1) {
					const innerTokens = tokens.slice(i + 1, closeIdx);
					const children = buildInlineFromLexTokens(innerTokens);
					const node: StrikethroughNode = {
						type: AstNodeType.Strikethrough,
						start: token.start,
						end: tokens[closeIdx].end,
						children
					};
					result.push(node);
					i = closeIdx + 1;
				} else {
					pushText(result, token);
					i++;
				}
				break;
			}

			case LexTokenKind.CodeSpanDelim: {
				const closeIdx = findClosingDelim(tokens, i + 1, LexTokenKind.CodeSpanDelim);
				if (closeIdx !== -1) {
					// Collect text content between code span delimiters
					let content = '';
					for (let j = i + 1; j < closeIdx; j++) {
						content += tokens[j].value;
					}
					const node: InlineCodeNode = {
						type: AstNodeType.InlineCode,
						start: token.start,
						end: tokens[closeIdx].end,
						content
					};
					result.push(node);
					i = closeIdx + 1;
				} else {
					pushText(result, token);
					i++;
				}
				break;
			}

			case LexTokenKind.LatexInlineOpen: {
				const closeIdx = findClosingDelim(tokens, i + 1, LexTokenKind.LatexInlineClose);
				if (closeIdx !== -1) {
					let content = '';
					for (let j = i + 1; j < closeIdx; j++) {
						content += tokens[j].value;
					}
					const node: LatexInlineNode = {
						type: AstNodeType.LatexInline,
						start: token.start,
						end: tokens[closeIdx].end,
						content
					};
					result.push(node);
					i = closeIdx + 1;
				} else {
					pushText(result, token);
					i++;
				}
				break;
			}

			case LexTokenKind.LatexInlineClose: {
				// Unmatched close — treat as text
				pushText(result, token);
				i++;
				break;
			}

			case LexTokenKind.LinkStart: {
				const closeIdx = findClosingDelim(tokens, i + 1, LexTokenKind.LinkEnd);
				if (closeIdx !== -1) {
					const innerTokens = tokens.slice(i + 1, closeIdx);
					// Re-lex link text for nested inline formatting
					const linkTextContent = innerTokens.map((t) => t.value).join('');
					const children =
						innerTokens.length > 0 ? buildInlineAst(linkTextContent, innerTokens[0].start) : [];
					const url = tokens[closeIdx].value;
					const node: LinkNode = {
						type: AstNodeType.Link,
						start: token.start,
						end: tokens[closeIdx].end,
						url,
						children
					};
					result.push(node);
					i = closeIdx + 1;
				} else {
					pushText(result, token);
					i++;
				}
				break;
			}

			case LexTokenKind.LinkEnd: {
				// Unmatched close — treat as text
				pushText(result, token);
				i++;
				break;
			}

			case LexTokenKind.ImageStart: {
				const parts = token.value.split('|');
				const alt = parts[0] || '';
				const url = parts.slice(1).join('|');
				const node: ImageNode = {
					type: AstNodeType.Image,
					start: token.start,
					end: token.end,
					url,
					alt
				};
				result.push(node);
				i++;
				break;
			}

			case LexTokenKind.LineBreak: {
				const node: LineBreakNode = {
					type: AstNodeType.LineBreak,
					start: token.start,
					end: token.end
				};
				result.push(node);
				i++;
				break;
			}

			case LexTokenKind.Text: {
				pushText(result, token);
				i++;
				break;
			}

			default: {
				// Any other token kind in inline context → treat as text
				pushText(result, token);
				i++;
				break;
			}
		}
	}

	return mergeTextNodes(result);
}

function findClosingDelim(tokens: LexToken[], startFrom: number, kind: LexTokenKind): number {
	for (let i = startFrom; i < tokens.length; i++) {
		if (tokens[i].kind === kind) {
			return i;
		}
	}
	return -1;
}

function pushText(result: AstNode[], token: LexToken): void {
	const node: TextNode = {
		type: AstNodeType.Text,
		start: token.start,
		end: token.end,
		content: token.value
	};
	result.push(node);
}

function mergeTextNodes(nodes: AstNode[]): AstNode[] {
	const merged: AstNode[] = [];
	let current: TextNode | null = null;

	for (const node of nodes) {
		if (node.type === AstNodeType.Text) {
			const textNode = node as TextNode;
			if (current) {
				current.content += textNode.content;
				current.end = textNode.end;
			} else {
				current = { ...textNode };
			}
		} else {
			if (current) {
				merged.push(current);
				current = null;
			}
			merged.push(node);
		}
	}

	if (current) {
		merged.push(current);
	}

	return merged;
}
