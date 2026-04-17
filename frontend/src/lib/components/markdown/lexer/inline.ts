import { LexTokenKind, type LexToken } from './types';

/**
 * Lex inline content within a text span, producing delimiter tokens
 * for bold, italic, code, latex, links, images, etc.
 *
 * Operates on a single text value (not the full source), so `baseOffset`
 * is added to all positions to keep them relative to the original source.
 */
export function lexInline(text: string, baseOffset: number): LexToken[] {
	const tokens: LexToken[] = [];
	let pos = 0;
	let textStart = 0;

	while (pos < text.length) {
		// Display math: \[ ... \]
		if (text[pos] === '\\' && text[pos + 1] === '[') {
			const endPos = text.indexOf('\\]', pos + 2);
			if (endPos !== -1) {
				flushText(tokens, text, textStart, pos, baseOffset);
				tokens.push({
					kind: LexTokenKind.LatexBlockOpen,
					start: baseOffset + pos,
					end: baseOffset + pos + 2,
					value: '\\['
				});
				const content = text.substring(pos + 2, endPos);
				if (content.length > 0) {
					tokens.push({
						kind: LexTokenKind.Text,
						start: baseOffset + pos + 2,
						end: baseOffset + endPos,
						value: content
					});
				}
				tokens.push({
					kind: LexTokenKind.LatexBlockClose,
					start: baseOffset + endPos,
					end: baseOffset + endPos + 2,
					value: '\\]'
				});
				pos = endPos + 2;
				textStart = pos;
				continue;
			}
		}

		// LaTeX inline: \( ... \)
		if (text[pos] === '\\' && text[pos + 1] === '(') {
			const endPos = text.indexOf('\\)', pos + 2);
			if (endPos !== -1) {
				flushText(tokens, text, textStart, pos, baseOffset);
				tokens.push({
					kind: LexTokenKind.LatexInlineOpen,
					start: baseOffset + pos,
					end: baseOffset + pos + 2,
					value: '\\('
				});
				const content = text.substring(pos + 2, endPos);
				if (content.length > 0) {
					tokens.push({
						kind: LexTokenKind.Text,
						start: baseOffset + pos + 2,
						end: baseOffset + endPos,
						value: content
					});
				}
				tokens.push({
					kind: LexTokenKind.LatexInlineClose,
					start: baseOffset + endPos,
					end: baseOffset + endPos + 2,
					value: '\\)'
				});
				pos = endPos + 2;
				textStart = pos;
				continue;
			}
		}

		// Inline display math: $$...$$ (must precede single-$ check)
		if (text[pos] === '$' && text[pos + 1] === '$') {
			const endPos = text.indexOf('$$', pos + 2);
			if (endPos !== -1) {
				const content = text.substring(pos + 2, endPos);
				if (content.length > 0) {
					flushText(tokens, text, textStart, pos, baseOffset);
					tokens.push({
						kind: LexTokenKind.LatexInlineOpen,
						start: baseOffset + pos,
						end: baseOffset + pos + 2,
						value: '$$'
					});
					tokens.push({
						kind: LexTokenKind.Text,
						start: baseOffset + pos + 2,
						end: baseOffset + endPos,
						value: content
					});
					tokens.push({
						kind: LexTokenKind.LatexInlineClose,
						start: baseOffset + endPos,
						end: baseOffset + endPos + 2,
						value: '$$'
					});
					pos = endPos + 2;
					textStart = pos;
					continue;
				}
			}
		}

		// Inline LaTeX: $...$  (not $$)
		if (text[pos] === '$' && text[pos + 1] !== '$') {
			const endPos = text.indexOf('$', pos + 1);
			if (endPos !== -1) {
				const content = text.substring(pos + 1, endPos);
				if (content.length > 0 && isValidInlineLatex(text, pos, endPos, content)) {
					flushText(tokens, text, textStart, pos, baseOffset);
					tokens.push({
						kind: LexTokenKind.LatexInlineOpen,
						start: baseOffset + pos,
						end: baseOffset + pos + 1,
						value: '$'
					});
					tokens.push({
						kind: LexTokenKind.Text,
						start: baseOffset + pos + 1,
						end: baseOffset + endPos,
						value: content
					});
					tokens.push({
						kind: LexTokenKind.LatexInlineClose,
						start: baseOffset + endPos,
						end: baseOffset + endPos + 1,
						value: '$'
					});
					pos = endPos + 1;
					textStart = pos;
					continue;
				}
			}
		}

		// Image: ![alt](url)
		if (text[pos] === '!' && text[pos + 1] === '[') {
			const match = text.substring(pos).match(/^!\[([^\]]*)\]\(([^)]+)\)/);
			if (match) {
				flushText(tokens, text, textStart, pos, baseOffset);
				tokens.push({
					kind: LexTokenKind.ImageStart,
					start: baseOffset + pos,
					end: baseOffset + pos + match[0].length,
					value: match[1] + '|' + match[2]
				});
				pos += match[0].length;
				textStart = pos;
				continue;
			}
		}

		// Link: [text](url) or <url>
		if (text[pos] === '<') {
			const match = text.substring(pos).match(/^<(https?:\/\/[^>]+)>/);
			if (match) {
				flushText(tokens, text, textStart, pos, baseOffset);
				tokens.push({
					kind: LexTokenKind.LinkStart,
					start: baseOffset + pos,
					end: baseOffset + pos + 1,
					value: '['
				});
				tokens.push({
					kind: LexTokenKind.Text,
					start: baseOffset + pos + 1,
					end: baseOffset + pos + 1 + match[1].length,
					value: match[1]
				});
				tokens.push({
					kind: LexTokenKind.LinkEnd,
					start: baseOffset + pos + 1 + match[1].length,
					end: baseOffset + pos + match[0].length,
					value: match[1]
				});
				pos += match[0].length;
				textStart = pos;
				continue;
			}
		}

		if (text[pos] === '[') {
			const match = text.substring(pos).match(/^\[([^\]]+)\]\(([^)]+)\)/);
			if (match) {
				flushText(tokens, text, textStart, pos, baseOffset);
				tokens.push({
					kind: LexTokenKind.LinkStart,
					start: baseOffset + pos,
					end: baseOffset + pos + 1,
					value: '['
				});
				// Link text content — will be inline-lexed recursively by the parser
				tokens.push({
					kind: LexTokenKind.Text,
					start: baseOffset + pos + 1,
					end: baseOffset + pos + 1 + match[1].length,
					value: match[1]
				});
				tokens.push({
					kind: LexTokenKind.LinkEnd,
					start: baseOffset + pos + 1 + match[1].length,
					end: baseOffset + pos + match[0].length,
					value: match[2]
				});
				pos += match[0].length;
				textStart = pos;
				continue;
			}
		}

		// Inline code: `...`
		if (text[pos] === '`') {
			const endPos = text.indexOf('`', pos + 1);
			if (endPos !== -1) {
				flushText(tokens, text, textStart, pos, baseOffset);
				tokens.push({
					kind: LexTokenKind.CodeSpanDelim,
					start: baseOffset + pos,
					end: baseOffset + pos + 1,
					value: '`'
				});
				const content = text.substring(pos + 1, endPos);
				if (content.length > 0) {
					tokens.push({
						kind: LexTokenKind.Text,
						start: baseOffset + pos + 1,
						end: baseOffset + endPos,
						value: content
					});
				}
				tokens.push({
					kind: LexTokenKind.CodeSpanDelim,
					start: baseOffset + endPos,
					end: baseOffset + endPos + 1,
					value: '`'
				});
				pos = endPos + 1;
				textStart = pos;
				continue;
			}
		}

		// Bold+Italic combined: *** or ___
		if (
			(text[pos] === '*' && text[pos + 1] === '*' && text[pos + 2] === '*') ||
			(text[pos] === '_' && text[pos + 1] === '_' && text[pos + 2] === '_')
		) {
			const ch = text[pos];
			const tripleDelim = ch + ch + ch;
			const endPos = text.indexOf(tripleDelim, pos + 3);
			if (endPos !== -1) {
				flushText(tokens, text, textStart, pos, baseOffset);
				tokens.push({
					kind: LexTokenKind.BoldDelim,
					start: baseOffset + pos,
					end: baseOffset + pos + 2,
					value: ch + ch
				});
				tokens.push({
					kind: LexTokenKind.ItalicDelim,
					start: baseOffset + pos + 2,
					end: baseOffset + pos + 3,
					value: ch
				});
				const content = text.substring(pos + 3, endPos);
				if (content.length > 0) {
					const innerTokens = lexInline(content, baseOffset + pos + 3);
					tokens.push(...innerTokens);
				}
				tokens.push({
					kind: LexTokenKind.ItalicDelim,
					start: baseOffset + endPos,
					end: baseOffset + endPos + 1,
					value: ch
				});
				tokens.push({
					kind: LexTokenKind.BoldDelim,
					start: baseOffset + endPos + 1,
					end: baseOffset + endPos + 3,
					value: ch + ch
				});
				pos = endPos + 3;
				textStart = pos;
				continue;
			}
			// No closing *** — fall through to bold check
		}

		// Bold: ** or __
		if (
			(text[pos] === '*' && text[pos + 1] === '*' && text[pos + 2] !== '*') ||
			(text[pos] === '_' && text[pos + 1] === '_' && text[pos + 2] !== '_')
		) {
			const delim = text.substring(pos, pos + 2);
			const endPos = findBoldClose(text, pos + 2, delim);
			if (endPos !== -1) {
				flushText(tokens, text, textStart, pos, baseOffset);
				tokens.push({
					kind: LexTokenKind.BoldDelim,
					start: baseOffset + pos,
					end: baseOffset + pos + 2,
					value: delim
				});
				// Content between delimiters — will be recursively lexed
				const content = text.substring(pos + 2, endPos);
				if (content.length > 0) {
					const innerTokens = lexInline(content, baseOffset + pos + 2);
					tokens.push(...innerTokens);
				}
				tokens.push({
					kind: LexTokenKind.BoldDelim,
					start: baseOffset + endPos,
					end: baseOffset + endPos + 2,
					value: delim
				});
				pos = endPos + 2;
				textStart = pos;
				continue;
			}
		}

		// Strikethrough: ~~
		if (text[pos] === '~' && text[pos + 1] === '~') {
			const endPos = text.indexOf('~~', pos + 2);
			if (endPos !== -1) {
				flushText(tokens, text, textStart, pos, baseOffset);
				tokens.push({
					kind: LexTokenKind.StrikethroughDelim,
					start: baseOffset + pos,
					end: baseOffset + pos + 2,
					value: '~~'
				});
				const content = text.substring(pos + 2, endPos);
				if (content.length > 0) {
					const innerTokens = lexInline(content, baseOffset + pos + 2);
					tokens.push(...innerTokens);
				}
				tokens.push({
					kind: LexTokenKind.StrikethroughDelim,
					start: baseOffset + endPos,
					end: baseOffset + endPos + 2,
					value: '~~'
				});
				pos = endPos + 2;
				textStart = pos;
				continue;
			}
		}

		// Italic: * or _ (single, not double)
		if (
			(text[pos] === '*' && text[pos + 1] !== '*') ||
			(text[pos] === '_' && text[pos + 1] !== '_')
		) {
			const delim = text[pos];
			const endPos = findItalicClose(text, pos + 1, delim);
			if (endPos !== -1) {
				flushText(tokens, text, textStart, pos, baseOffset);
				tokens.push({
					kind: LexTokenKind.ItalicDelim,
					start: baseOffset + pos,
					end: baseOffset + pos + 1,
					value: delim
				});
				const content = text.substring(pos + 1, endPos);
				if (content.length > 0) {
					const innerTokens = lexInline(content, baseOffset + pos + 1);
					tokens.push(...innerTokens);
				}
				tokens.push({
					kind: LexTokenKind.ItalicDelim,
					start: baseOffset + endPos,
					end: baseOffset + endPos + 1,
					value: delim
				});
				pos = endPos + 1;
				textStart = pos;
				continue;
			}
		}

		// Line break: <br>, <br/>, <br />, trailing spaces + newline, or single newline
		if (text[pos] === '<') {
			const brMatch = text.substring(pos).match(/^<br\s*\/?>/i);
			if (brMatch) {
				flushText(tokens, text, textStart, pos, baseOffset);
				tokens.push({
					kind: LexTokenKind.LineBreak,
					start: baseOffset + pos,
					end: baseOffset + pos + brMatch[0].length,
					value: brMatch[0]
				});
				pos += brMatch[0].length;
				textStart = pos;
				continue;
			}
		}

		// Trailing spaces + newline
		if (text[pos] === ' ') {
			const trailingMatch = text.substring(pos).match(/^( {2,})(\r?\n)/);
			if (trailingMatch) {
				flushText(tokens, text, textStart, pos, baseOffset);
				tokens.push({
					kind: LexTokenKind.LineBreak,
					start: baseOffset + pos,
					end: baseOffset + pos + trailingMatch[0].length,
					value: trailingMatch[0]
				});
				pos += trailingMatch[0].length;
				textStart = pos;
				continue;
			}
		}

		// Single newline within inline content
		if (text[pos] === '\r' || text[pos] === '\n') {
			const nlMatch = text.substring(pos).match(/^\r?\n/);
			if (nlMatch) {
				flushText(tokens, text, textStart, pos, baseOffset);
				tokens.push({
					kind: LexTokenKind.LineBreak,
					start: baseOffset + pos,
					end: baseOffset + pos + nlMatch[0].length,
					value: nlMatch[0]
				});
				pos += nlMatch[0].length;
				textStart = pos;
				continue;
			}
		}

		pos++;
	}

	// Flush remaining text
	flushText(tokens, text, textStart, pos, baseOffset);

	return tokens;
}

function flushText(
	tokens: LexToken[],
	text: string,
	start: number,
	end: number,
	baseOffset: number
): void {
	if (end > start) {
		tokens.push({
			kind: LexTokenKind.Text,
			start: baseOffset + start,
			end: baseOffset + end,
			value: text.substring(start, end)
		});
	}
}

function findItalicClose(text: string, startFrom: number, delim: string): number {
	for (let i = startFrom; i < text.length; i++) {
		if (text[i] === delim) {
			// For *, make sure next char isn't also *  (that would be bold)
			if (delim === '*' && text[i + 1] === '*') continue;
			if (delim === '_' && text[i + 1] === '_') continue;
			return i;
		}
	}
	return -1;
}

function findBoldClose(text: string, startFrom: number, delim: string): number {
	const ch = delim[0];
	for (let i = startFrom; i < text.length - 1; i++) {
		if (text[i] === ch && text[i + 1] === ch) {
			// When encountering a run of 3+ stars, the closing ** is the last
			// two in the run — skip to the end of the run.
			let runEnd = i + 2;
			while (runEnd < text.length && text[runEnd] === ch) runEnd++;
			const runLen = runEnd - i;
			if (runLen >= 3) {
				// Use the last two stars of this run as the bold close
				return runEnd - 2;
			}
			return i;
		}
	}
	return -1;
}

function isValidInlineLatex(text: string, pos: number, endPos: number, content: string): boolean {
	if (content.length === 0) return false;

	const startsWithSpace = content[0] === ' ';
	const endsWithSpace = content[content.length - 1] === ' ';
	if (startsWithSpace !== endsWithSpace) return false;

	const hasSpaces = /\s/.test(content);
	if (hasSpaces) {
		if (pos > 0 && text[pos - 1] !== ' ') return false;
		const closingPos = endPos + 1;
		if (closingPos < text.length && text[closingPos] !== ' ') return false;
	}

	return true;
}
