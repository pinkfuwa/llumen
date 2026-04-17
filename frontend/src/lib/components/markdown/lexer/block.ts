import { LexTokenKind, type LexToken } from './types';

function peekLine(source: string, pos: number): string {
	const nl = source.indexOf('\n', pos);
	return nl === -1 ? source.substring(pos) : source.substring(pos, nl);
}

function isBlankLine(line: string): boolean {
	return line.trim().length === 0;
}

function isHeading(line: string): RegExpMatchArray | null {
	return line.match(/^(#{1,6})\s+(.+)$/);
}

function isFenceOpen(line: string): RegExpMatchArray | null {
	return line.match(/^(\s*)```(\w*)$/);
}

function isFenceClose(line: string): boolean {
	return /^\s*```\s*$/.test(line);
}

function isHorizontalRule(line: string): boolean {
	return /^---+$/.test(line) || /^\*\*\*+$/.test(line) || /^___+$/.test(line);
}

function isBlockquoteStart(line: string): boolean {
	return line.startsWith('>');
}

function isOrderedListItem(line: string): RegExpMatchArray | null {
	return line.match(/^(\d+)\.\s+(.*)$/);
}

function isUnorderedListItem(line: string): RegExpMatchArray | null {
	return line.match(/^([-*+])\s+(.*)$/);
}

function isIndentedListItem(line: string): RegExpMatchArray | null {
	return line.match(/^(\s+)([-*+])\s+(.*)$/);
}

function isTableSeparator(line: string): boolean {
	const trimmed = line.trim();
	if (!trimmed.includes('|')) return false;
	return /^[\|\s]*:?-+:?[\|\s:]*(:?-+:?[\|\s]*)*$/.test(trimmed);
}

function isLatexBlockOpen(source: string, pos: number): { delimiter: string; skip: number } | null {
	if (source.substring(pos, pos + 2) === '\\[') {
		return { delimiter: '\\[', skip: 2 };
	}
	const line = peekLine(source, pos);
	if (line.trimStart().startsWith('$$')) {
		const trimmedOffset = line.length - line.trimStart().length;
		return { delimiter: '$$', skip: trimmedOffset + 2 };
	}
	return null;
}

/**
 * Lex block-level structure of markdown source.
 *
 * This operates line-by-line and produces flat tokens that mark
 * structural boundaries (headings, fences, pipes, etc.) along with
 * Text spans for everything that isn't structural.
 *
 * Inline delimiters (bold, italic, etc.) are NOT handled here — they
 * are processed by the inline lexer on the Text content.
 */
export function lexBlock(source: string): LexToken[] {
	const tokens: LexToken[] = [];
	let pos = 0;

	while (pos < source.length) {
		const line = peekLine(source, pos);
		const lineEnd = pos + line.length;
		const afterLine = lineEnd < source.length ? lineEnd + 1 : lineEnd; // skip \n

		// Blank line
		if (isBlankLine(line)) {
			tokens.push({ kind: LexTokenKind.BlankLine, start: pos, end: afterLine, value: '' });
			pos = afterLine;
			continue;
		}

		// Heading
		const headingMatch = isHeading(line);
		if (headingMatch) {
			const level = headingMatch[1].length;
			const markerEnd = pos + headingMatch[1].length + 1; // # + space
			tokens.push({
				kind: LexTokenKind.HeadingMarker,
				start: pos,
				end: markerEnd,
				value: String(level)
			});
			// The heading content is Text that will be inline-lexed later
			tokens.push({
				kind: LexTokenKind.Text,
				start: markerEnd,
				end: lineEnd,
				value: headingMatch[2]
			});
			if (afterLine > lineEnd) {
				tokens.push({
					kind: LexTokenKind.Newline,
					start: lineEnd,
					end: afterLine,
					value: '\n'
				});
			}
			pos = afterLine;
			continue;
		}

		// Fence open
		const fenceMatch = isFenceOpen(line);
		if (fenceMatch) {
			tokens.push({
				kind: LexTokenKind.FenceOpen,
				start: pos,
				end: afterLine,
				value: fenceMatch[2] || ''
			});
			pos = afterLine;

			// Consume everything until fence close or end of source
			let closed = false;
			const contentStart = pos;
			while (pos < source.length) {
				const innerLine = peekLine(source, pos);
				const innerEnd = pos + innerLine.length;
				const innerAfter = innerEnd < source.length ? innerEnd + 1 : innerEnd;

				if (isFenceClose(innerLine)) {
					// Emit content before the close fence
					if (pos > contentStart) {
						tokens.push({
							kind: LexTokenKind.Text,
							start: contentStart,
							end: pos,
							value: source.substring(contentStart, pos)
						});
					}
					tokens.push({
						kind: LexTokenKind.FenceClose,
						start: pos,
						end: innerAfter,
						value: '```'
					});
					pos = innerAfter;
					closed = true;
					break;
				}
				pos = innerAfter;
			}

			// Unclosed fence — emit remaining content
			if (!closed && pos > contentStart) {
				tokens.push({
					kind: LexTokenKind.Text,
					start: contentStart,
					end: pos,
					value: source.substring(contentStart, pos)
				});
			}
			continue;
		}

		// LaTeX block
		const latexOpen = isLatexBlockOpen(source, pos);
		if (latexOpen) {
			const delimStart = pos;
			const afterDelim = pos + latexOpen.skip;
			const endDelim = latexOpen.delimiter === '\\[' ? '\\]' : '$$';

			const endPos = source.indexOf(endDelim, afterDelim);
			if (endPos !== -1) {
				tokens.push({
					kind: LexTokenKind.LatexBlockOpen,
					start: delimStart,
					end: afterDelim,
					value: latexOpen.delimiter
				});

				// Content between delimiters
				const content = source.substring(afterDelim, endPos).trim();
				if (content.length > 0) {
					tokens.push({
						kind: LexTokenKind.Text,
						start: afterDelim,
						end: endPos,
						value: source.substring(afterDelim, endPos)
					});
				}

				const closeEnd = endPos + endDelim.length;
				tokens.push({
					kind: LexTokenKind.LatexBlockClose,
					start: endPos,
					end: closeEnd,
					value: endDelim
				});

				// Skip past any trailing newline
				pos = closeEnd;
				if (pos < source.length && source[pos] === '\n') pos++;
				continue;
			}
			// No closing delimiter — fall through to treat as text
		}

		// Horizontal rule
		if (isHorizontalRule(line)) {
			tokens.push({
				kind: LexTokenKind.HorizontalRule,
				start: pos,
				end: afterLine,
				value: line
			});
			pos = afterLine;
			continue;
		}

		// Table separator row (must be checked before generic table pipe)
		if (isTableSeparator(line)) {
			tokens.push({
				kind: LexTokenKind.TableSeparatorRow,
				start: pos,
				end: afterLine,
				value: line
			});
			pos = afterLine;
			continue;
		}

		// Blockquote
		if (isBlockquoteStart(line)) {
			tokens.push({
				kind: LexTokenKind.BlockquoteMarker,
				start: pos,
				end: pos + 1,
				value: '>'
			});
			// Content after "> " is Text (skip the > and optional space)
			let contentStart = pos + 1;
			if (source[contentStart] === ' ') contentStart++;
			if (contentStart < lineEnd) {
				tokens.push({
					kind: LexTokenKind.Text,
					start: contentStart,
					end: lineEnd,
					value: source.substring(contentStart, lineEnd)
				});
			}
			if (afterLine > lineEnd) {
				tokens.push({
					kind: LexTokenKind.Newline,
					start: lineEnd,
					end: afterLine,
					value: '\n'
				});
			}
			pos = afterLine;
			continue;
		}

		// List items (ordered)
		const orderedMatch = isOrderedListItem(line);
		if (orderedMatch) {
			const markerLen = orderedMatch[1].length + 2; // "1. "
			tokens.push({
				kind: LexTokenKind.ListItemMarker,
				start: pos,
				end: pos + markerLen,
				value: orderedMatch[1] + '. '
			});
			if (orderedMatch[2].length > 0) {
				tokens.push({
					kind: LexTokenKind.Text,
					start: pos + markerLen,
					end: lineEnd,
					value: orderedMatch[2]
				});
			}
			if (afterLine > lineEnd) {
				tokens.push({
					kind: LexTokenKind.Newline,
					start: lineEnd,
					end: afterLine,
					value: '\n'
				});
			}
			pos = afterLine;
			continue;
		}

		// List items (unordered)
		const unorderedMatch = isUnorderedListItem(line);
		if (unorderedMatch) {
			const markerLen = 2; // "- "
			tokens.push({
				kind: LexTokenKind.ListItemMarker,
				start: pos,
				end: pos + markerLen,
				value: unorderedMatch[1] + ' '
			});
			if (unorderedMatch[2].length > 0) {
				tokens.push({
					kind: LexTokenKind.Text,
					start: pos + markerLen,
					end: lineEnd,
					value: unorderedMatch[2]
				});
			}
			if (afterLine > lineEnd) {
				tokens.push({
					kind: LexTokenKind.Newline,
					start: lineEnd,
					end: afterLine,
					value: '\n'
				});
			}
			pos = afterLine;
			continue;
		}

		// Indented list items
		const indentedMatch = isIndentedListItem(line);
		if (indentedMatch) {
			const indent = indentedMatch[1];
			const markerLen = indent.length + 2; // indent + "- "
			tokens.push({
				kind: LexTokenKind.ListItemMarker,
				start: pos,
				end: pos + markerLen,
				value: indent + indentedMatch[2] + ' '
			});
			if (indentedMatch[3].length > 0) {
				tokens.push({
					kind: LexTokenKind.Text,
					start: pos + markerLen,
					end: lineEnd,
					value: indentedMatch[3]
				});
			}
			if (afterLine > lineEnd) {
				tokens.push({
					kind: LexTokenKind.Newline,
					start: lineEnd,
					end: afterLine,
					value: '\n'
				});
			}
			pos = afterLine;
			continue;
		}

		// Check if line looks like a table row (has pipes)
		if (looksLikeTableRow(line)) {
			lexTableRow(tokens, source, pos, line, lineEnd, afterLine);
			pos = afterLine;
			continue;
		}

		// Default: plain text line
		tokens.push({
			kind: LexTokenKind.Text,
			start: pos,
			end: lineEnd,
			value: line
		});
		if (afterLine > lineEnd) {
			tokens.push({
				kind: LexTokenKind.Newline,
				start: lineEnd,
				end: afterLine,
				value: '\n'
			});
		}
		pos = afterLine;
	}

	return tokens;
}

function looksLikeTableRow(line: string): boolean {
	const trimmed = line.trim();
	if (trimmed.includes('`') && !trimmed.startsWith('|')) {
		return false;
	}
	if (!trimmed.startsWith('|')) {
		if (trimmed.includes('\\(') || trimmed.includes('\\)')) {
			return false;
		}
	}
	const pipeCount = (trimmed.match(/\|/g) || []).length;
	return pipeCount >= 2 || trimmed.includes('\t');
}

/**
 * Lex a table row, emitting TablePipe tokens for structural pipes
 * and Text tokens for cell content. LaTeX delimiters (\(, \), $) are
 * tracked so that pipes inside LaTeX are emitted as Text, not TablePipe.
 */
function lexTableRow(
	tokens: LexToken[],
	source: string,
	lineStart: number,
	line: string,
	lineEnd: number,
	afterLine: number
): void {
	let i = 0;
	let textStart = lineStart;
	let latexDepth = 0;

	while (i < line.length) {
		// Track latex context so pipes inside latex are not treated as separators
		if (line.substring(i, i + 2) === '\\(' || line.substring(i, i + 2) === '\\[') {
			latexDepth++;
			i += 2;
			continue;
		}
		if (line.substring(i, i + 2) === '\\)' || line.substring(i, i + 2) === '\\]') {
			latexDepth = Math.max(0, latexDepth - 1);
			i += 2;
			continue;
		}
		if (latexDepth === 0 && line[i] === '$') {
			// Check for $$ or single $
			if (line[i + 1] === '$') {
				latexDepth++;
				i += 2;
				continue;
			}
			latexDepth++;
			i++;
			continue;
		}
		if (latexDepth > 0 && line[i] === '$') {
			if (line[i + 1] === '$') {
				latexDepth = Math.max(0, latexDepth - 1);
				i += 2;
				continue;
			}
			latexDepth = Math.max(0, latexDepth - 1);
			i++;
			continue;
		}

		if (latexDepth === 0 && line[i] === '|') {
			// Emit text before this pipe
			const pipePos = lineStart + i;
			if (pipePos > textStart) {
				tokens.push({
					kind: LexTokenKind.Text,
					start: textStart,
					end: pipePos,
					value: source.substring(textStart, pipePos)
				});
			}
			tokens.push({
				kind: LexTokenKind.TablePipe,
				start: pipePos,
				end: pipePos + 1,
				value: '|'
			});
			textStart = pipePos + 1;
			i++;
			continue;
		}

		i++;
	}

	// Emit remaining text after last pipe
	if (textStart < lineEnd) {
		const remaining = source.substring(textStart, lineEnd).trim();
		if (remaining.length > 0) {
			tokens.push({
				kind: LexTokenKind.Text,
				start: textStart,
				end: lineEnd,
				value: source.substring(textStart, lineEnd)
			});
		}
	}

	if (afterLine > lineEnd) {
		tokens.push({
			kind: LexTokenKind.Newline,
			start: lineEnd,
			end: afterLine,
			value: '\n'
		});
	}
}
