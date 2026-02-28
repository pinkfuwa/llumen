import { TokenType, type Token } from '../tokens';
import * as builders from '../tokens/builders';

export interface InlineParseResult {
	token: Token | null;
	newPosition: number;
}

export function parseInlineLatex(
	text: string,
	position: number,
	baseOffset: number
): InlineParseResult {
	const remaining = text.substring(position);

	if (remaining.startsWith('\\(')) {
		const endPos = remaining.indexOf('\\)', 2);
		if (endPos !== -1) {
			const content = remaining.substring(2, endPos);
			return {
				token: builders.createLatexInlineToken(
					content,
					baseOffset + position,
					baseOffset + position + endPos + 2
				),
				newPosition: position + endPos + 2
			};
		}
	}

	if (remaining.startsWith('$')) {
		const afterDollar = remaining.substring(1);

		if (afterDollar.startsWith('$')) {
			return { token: null, newPosition: position };
		}

		const endPos = afterDollar.indexOf('$');
		if (endPos === -1) {
			return { token: null, newPosition: position };
		}

		const content = afterDollar.substring(0, endPos);

		if (content.length === 0) {
			return { token: null, newPosition: position };
		}

		const startsWithSpace = content[0] === ' ';
		const endsWithSpace = content[content.length - 1] === ' ';

		if (startsWithSpace !== endsWithSpace) {
			return { token: null, newPosition: position };
		}

		const hasSpaces = /\s/.test(content);

		if (hasSpaces) {
			if (position > 0 && text[position - 1] !== ' ') {
				return { token: null, newPosition: position };
			}

			const closingPos = position + 1 + endPos + 1;
			if (closingPos < text.length && text[closingPos] !== ' ') {
				return { token: null, newPosition: position };
			}
		}

		return {
			token: builders.createLatexInlineToken(
				content,
				baseOffset + position,
				baseOffset + position + endPos + 2
			),
			newPosition: position + endPos + 2
		};
	}

	return { token: null, newPosition: position };
}

export function parseImage(text: string, position: number, baseOffset: number): InlineParseResult {
	const remaining = text.substring(position);
	const match = remaining.match(/^!\[([^\]]*)\]\(([^)]+)\)/);

	if (!match) {
		return { token: null, newPosition: position };
	}

	const alt = match[1];
	const url = match[2];

	return {
		token: builders.createImageToken(
			url,
			alt,
			baseOffset + position,
			baseOffset + position + match[0].length
		),
		newPosition: position + match[0].length
	};
}

export function parseLink(text: string, position: number, baseOffset: number): InlineParseResult {
	const remaining = text.substring(position);

	const angleBracketMatch = remaining.match(/^<(https?:\/\/[^>]+)>/);
	if (angleBracketMatch) {
		const url = angleBracketMatch[1];

		const textToken = builders.createTextToken(
			url,
			baseOffset + position + 1,
			baseOffset + position + 1 + url.length
		);

		return {
			token: builders.createLinkToken(
				url,
				[textToken],
				baseOffset + position,
				baseOffset + position + angleBracketMatch[0].length
			),
			newPosition: position + angleBracketMatch[0].length
		};
	}

	const match = remaining.match(/^\[([^\]]+)\]\(([^)]+)\)/);

	if (!match) {
		return { token: null, newPosition: position };
	}

	const linkText = match[1];
	const url = match[2];

	const inlineTokens = parseInline(linkText, baseOffset + position + 1);

	return {
		token: builders.createLinkToken(
			url,
			inlineTokens,
			baseOffset + position,
			baseOffset + position + match[0].length
		),
		newPosition: position + match[0].length
	};
}

export function parseInlineCode(
	text: string,
	position: number,
	baseOffset: number
): InlineParseResult {
	const remaining = text.substring(position);

	if (!remaining.startsWith('`')) {
		return { token: null, newPosition: position };
	}

	const endPos = remaining.indexOf('`', 1);
	if (endPos === -1) {
		return { token: null, newPosition: position };
	}

	const content = remaining.substring(1, endPos);

	return {
		token: builders.createInlineCodeToken(
			content,
			baseOffset + position,
			baseOffset + position + endPos + 1
		),
		newPosition: position + endPos + 1
	};
}

export function parseBold(text: string, position: number, baseOffset: number): InlineParseResult {
	const remaining = text.substring(position);

	if (remaining.startsWith('**')) {
		const endPos = remaining.indexOf('**', 2);
		if (endPos !== -1) {
			const content = remaining.substring(2, endPos);
			const inlineTokens = parseInline(content, baseOffset + position + 2);

			return {
				token: builders.createBoldToken(
					inlineTokens,
					baseOffset + position,
					baseOffset + position + endPos + 2
				),
				newPosition: position + endPos + 2
			};
		}
	}

	if (remaining.startsWith('__')) {
		const endPos = remaining.indexOf('__', 2);
		if (endPos !== -1) {
			const content = remaining.substring(2, endPos);
			const inlineTokens = parseInline(content, baseOffset + position + 2);

			return {
				token: builders.createBoldToken(
					inlineTokens,
					baseOffset + position,
					baseOffset + position + endPos + 2
				),
				newPosition: position + endPos + 2
			};
		}
	}

	return { token: null, newPosition: position };
}

export function parseItalic(text: string, position: number, baseOffset: number): InlineParseResult {
	const remaining = text.substring(position);

	if (remaining.startsWith('*') && !remaining.startsWith('**')) {
		const endPos = remaining.indexOf('*', 1);
		if (endPos !== -1 && remaining[endPos + 1] !== '*') {
			const content = remaining.substring(1, endPos);
			const inlineTokens = parseInline(content, baseOffset + position + 1);

			return {
				token: builders.createItalicToken(
					inlineTokens,
					baseOffset + position,
					baseOffset + position + endPos + 1
				),
				newPosition: position + endPos + 1
			};
		}
	}

	if (remaining.startsWith('_') && !remaining.startsWith('__')) {
		const endPos = remaining.indexOf('_', 1);
		if (endPos !== -1 && remaining[endPos + 1] !== '_') {
			const content = remaining.substring(1, endPos);
			const inlineTokens = parseInline(content, baseOffset + position + 1);

			return {
				token: builders.createItalicToken(
					inlineTokens,
					baseOffset + position,
					baseOffset + position + endPos + 1
				),
				newPosition: position + endPos + 1
			};
		}
	}

	return { token: null, newPosition: position };
}

export function parseStrikethrough(
	text: string,
	position: number,
	baseOffset: number
): InlineParseResult {
	const remaining = text.substring(position);

	if (remaining.startsWith('~~')) {
		const endPos = remaining.indexOf('~~', 2);
		if (endPos !== -1) {
			const content = remaining.substring(2, endPos);
			const inlineTokens = parseInline(content, baseOffset + position + 2);

			return {
				token: builders.createStrikethroughToken(
					inlineTokens,
					baseOffset + position,
					baseOffset + position + endPos + 2
				),
				newPosition: position + endPos + 2
			};
		}
	}

	return { token: null, newPosition: position };
}

export function parseLineBreak(
	text: string,
	position: number,
	baseOffset: number
): InlineParseResult {
	const remaining = text.substring(position);

	const brMatch = remaining.match(/^<br\s*\/?>/i);
	if (brMatch) {
		return {
			token: builders.createLineBreakToken(
				baseOffset + position,
				baseOffset + position + brMatch[0].length
			),
			newPosition: position + brMatch[0].length
		};
	}

	const match = remaining.match(/^( {2,})(\r?\n)/);
	if (match) {
		return {
			token: builders.createLineBreakToken(
				baseOffset + position,
				baseOffset + position + match[0].length
			),
			newPosition: position + match[0].length
		};
	}

	const singleNewlineMatch = remaining.match(/^\r?\n/);
	if (singleNewlineMatch) {
		return {
			token: builders.createLineBreakToken(
				baseOffset + position,
				baseOffset + position + singleNewlineMatch[0].length
			),
			newPosition: position + singleNewlineMatch[0].length
		};
	}

	return { token: null, newPosition: position };
}

function mergeTextTokens(tokens: Token[]): Token[] {
	const merged: Token[] = [];
	let currentText: Token | null = null;

	for (const token of tokens) {
		if (token.type === TokenType.Text) {
			if (currentText) {
				(currentText as typeof currentText & { content: string }).content += (
					token as typeof token & { content: string }
				).content;
				currentText.end = token.end;
			} else {
				currentText = { ...token };
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

export function parseInline(text: string, baseOffset: number = 0): Token[] {
	const foundTokens: Token[] = [];
	let position = 0;

	while (position < text.length) {
		const latexInline = parseInlineLatex(text, position, baseOffset);
		if (latexInline.token) {
			foundTokens.push(latexInline.token);
			position = latexInline.newPosition;
			continue;
		}

		const image = parseImage(text, position, baseOffset);
		if (image.token) {
			foundTokens.push(image.token);
			position = image.newPosition;
			continue;
		}

		const link = parseLink(text, position, baseOffset);
		if (link.token) {
			foundTokens.push(link.token);
			position = link.newPosition;
			continue;
		}

		const inlineCode = parseInlineCode(text, position, baseOffset);
		if (inlineCode.token) {
			foundTokens.push(inlineCode.token);
			position = inlineCode.newPosition;
			continue;
		}

		const bold = parseBold(text, position, baseOffset);
		if (bold.token) {
			foundTokens.push(bold.token);
			position = bold.newPosition;
			continue;
		}

		const italic = parseItalic(text, position, baseOffset);
		if (italic.token) {
			foundTokens.push(italic.token);
			position = italic.newPosition;
			continue;
		}

		const strikethrough = parseStrikethrough(text, position, baseOffset);
		if (strikethrough.token) {
			foundTokens.push(strikethrough.token);
			position = strikethrough.newPosition;
			continue;
		}

		const lineBreak = parseLineBreak(text, position, baseOffset);
		if (lineBreak.token) {
			foundTokens.push(lineBreak.token);
			position = lineBreak.newPosition;
			continue;
		}

		position++;
	}

	const tokens: Token[] = [];
	let currentPos = 0;

	for (const token of foundTokens) {
		const tokenStart = token.start - baseOffset;

		if (tokenStart > currentPos) {
			const textContent = text.substring(currentPos, tokenStart);
			tokens.push(builders.createTextToken(textContent, baseOffset + currentPos, token.start));
		}

		tokens.push(token);
		currentPos = token.end - baseOffset;
	}

	if (currentPos < text.length) {
		const textContent = text.substring(currentPos);
		tokens.push(
			builders.createTextToken(textContent, baseOffset + currentPos, baseOffset + text.length)
		);
	}

	if (tokens.length === 0) {
		tokens.push(builders.createTextToken(text, baseOffset, baseOffset + text.length));
	}

	return mergeTextTokens(tokens);
}
