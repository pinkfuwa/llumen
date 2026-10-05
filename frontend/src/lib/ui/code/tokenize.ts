import { tokenizeWith } from '@speed-highlight/core/tokenize';
import * as grammars from '@speed-highlight/core/languages';
import type { CodeLanguage } from './languages';
import type { CodeToken } from './types';

const languages = { ...grammars, 'leanpub-md': grammars.leanpubMd };

export function tokenizeCode(text: string, language: CodeLanguage): CodeToken[] {
	const tokens: CodeToken[] = [];
	let offset = 0;
	tokenizeWith(
		text,
		language,
		(content, type) => {
			const end = offset + content.length;
			if (type && end > offset) tokens.push({ start: offset, end, type });
			offset = end;
		},
		{ languages }
	);
	return tokens;
}
