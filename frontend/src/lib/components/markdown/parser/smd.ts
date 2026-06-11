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
	NEWLINE,
	MAYBE_URL,
	MAYBE_TASK,
	MAYBE_BR,
	MAYBE_EQ_BLOCK,
	MAYBE_LINK,
	HREF,
	SRC,
	LANG,
	CHECKED,
	START,
	type Parser,
	type Renderer,
	type RendererData
} from './types';

const TOKEN_ARRAY_CAP = 24;

export function level_to_heading(level: number): number {
	switch (level) {
		case 1:
			return HEADING_1;
		case 2:
			return HEADING_2;
		case 3:
			return HEADING_3;
		case 4:
			return HEADING_4;
		case 5:
			return HEADING_5;
		default:
			return HEADING_6;
	}
}

export function heading_to_level(token: number): number {
	switch (token) {
		case HEADING_1:
			return 1;
		case HEADING_2:
			return 2;
		case HEADING_3:
			return 3;
		case HEADING_4:
			return 4;
		case HEADING_5:
			return 5;
		case HEADING_6:
			return 6;
		default:
			return 0;
	}
}

export function parser(renderer: Renderer): Parser {
	const tokens = new Uint32Array(TOKEN_ARRAY_CAP);
	tokens[0] = DOCUMENT;
	return {
		renderer,
		text: '',
		pending: '',
		tokens,
		len: 0,
		token: DOCUMENT,
		fence_end: 0,
		blockquote_idx: 0,
		hr_char: '',
		hr_chars: 0,
		fence_start: 0,
		spaces: new Uint8Array(TOKEN_ARRAY_CAP),
		indent: '',
		indent_len: 0,
		table_state: 0,
		eq_open: 0,
		maybe_link_text: ''
	};
}

export function parser_end(p: Parser): void {
	if (p.pending.length > 0) {
		parser_write(p, '\n');
	}
}

function add_text(p: Parser): void {
	if (p.text.length === 0) return;
	p.renderer.add_text(p.renderer.data, p.text);
	p.text = '';
}

function ensure_paragraph(p: Parser): void {
	switch (p.token) {
		case LINE_BREAK:
		case DOCUMENT:
		case BLOCKQUOTE:
		case LIST_ORDERED:
		case LIST_UNORDERED:
			add_token(p, PARAGRAPH);
	}
}

function push_text(p: Parser, text: string): void {
	ensure_paragraph(p);
	p.text += text;
}

function end_token(p: Parser): void {
	p.len -= 1;
	p.token = p.tokens[p.len];
	p.renderer.end_token(p.renderer.data);
}

function add_token(p: Parser, token: number): void {
	if (
		(p.tokens[p.len] === LIST_ORDERED || p.tokens[p.len] === LIST_UNORDERED) &&
		token !== LIST_ITEM
	) {
		end_token(p);
	}

	p.len += 1;
	p.tokens[p.len] = token;
	p.token = token;
	p.renderer.add_token(p.renderer.data, token);
}

function idx_of_token(p: Parser, token: number, start_idx: number): number {
	while (start_idx <= p.len) {
		if (p.tokens[start_idx] === token) {
			return start_idx;
		}
		start_idx += 1;
	}
	return -1;
}

function end_tokens_to_len(p: Parser, len: number): void {
	p.fence_start = 0;
	while (p.len > len) {
		end_token(p);
	}
}

function end_tokens_to_indent(p: Parser, indent: number): number {
	let idx = 0;
	for (let i = 0; i <= p.len; i += 1) {
		indent -= p.spaces[i];
		if (indent < 0) {
			break;
		}
		switch (p.tokens[i]) {
			case CODE_BLOCK:
			case CODE_FENCE:
			case BLOCKQUOTE:
			case LIST_ITEM:
				idx = i;
				break;
		}
	}

	while (p.len > idx) {
		end_token(p);
	}

	return indent;
}

function continue_or_add_list(p: Parser, list_token: number): boolean {
	let list_idx = -1;
	let item_idx = -1;

	for (let i = p.blockquote_idx + 1; i <= p.len; i += 1) {
		if (p.tokens[i] === LIST_ITEM) {
			if (p.indent_len < p.spaces[i]) {
				item_idx = -1;
				break;
			}
			item_idx = i;
		} else if (p.tokens[i] === list_token) {
			list_idx = i;
		}
	}

	if (item_idx === -1) {
		if (list_idx === -1) {
			end_tokens_to_len(p, p.blockquote_idx);
			add_token(p, list_token);
			return true;
		}
		end_tokens_to_len(p, list_idx);
		return false;
	}
	end_tokens_to_len(p, item_idx);
	add_token(p, list_token);
	return true;
}

function add_list_item(p: Parser, prefix_length: number): void {
	add_token(p, LIST_ITEM);
	p.spaces[p.len] = p.indent_len + prefix_length;
	clear_root_pending(p);
	p.token = MAYBE_TASK;
}

function clear_root_pending(p: Parser): void {
	p.indent = '';
	p.indent_len = 0;
	p.pending = '';
}

function is_digit(charcode: number): boolean {
	switch (charcode) {
		case 48:
		case 49:
		case 50:
		case 51:
		case 52:
		case 53:
		case 54:
		case 55:
		case 56:
		case 57:
			return true;
		default:
			return false;
	}
}

function is_alphanumeric(ch: string): boolean {
	const code = ch.charCodeAt(0);
	return (code >= 48 && code <= 57) || (code >= 65 && code <= 90) || (code >= 97 && code <= 122);
}

function is_delimeter(charcode: number): boolean {
	switch (charcode) {
		case 32:
		case 58:
		case 59:
		case 41:
		case 44:
		case 33:
		case 46:
		case 63:
		case 93:
		case 10:
			return true;
		default:
			return false;
	}
}

function is_delimeter_or_number(charcode: number): boolean {
	return is_digit(charcode) || is_delimeter(charcode);
}

function is_block_eq(token: number): boolean {
	return (
		token === EQUATION_BLOCK || token === EQUATION_BLOCK_DOLLAR || token === EQUATION_BLOCK_BRACKET
	);
}

function indent_width(s: string): number {
	let w = 0;
	for (const c of s) {
		if (c === '\t') w += 4;
		else if (c === ' ') w += 1;
		else break;
	}
	return w;
}

export function parser_write(p: Parser, chunk: string): void {
	for (const char of chunk) {
		if (p.token === NEWLINE) {
			switch (char) {
				case ' ':
					p.indent_len += 1;
					continue;
				case '\t':
					p.indent_len += 4;
					continue;
			}

			let indent = end_tokens_to_indent(p, p.indent_len);

			p.indent_len = 0;
			p.token = p.tokens[p.len];

			if (indent > 0) {
				parser_write(p, ' '.repeat(indent));
			}
		}

		const pending_with_char = p.pending + char;

		switch (p.token) {
			case LINE_BREAK:
			case DOCUMENT:
			case BLOCKQUOTE:
			case LIST_ORDERED:
			case LIST_UNORDERED:
				switch (p.pending[0]) {
					case undefined:
						p.pending = char;
						continue;
					case ' ':
						p.pending = char;
						p.indent += ' ';
						p.indent_len += 1;
						continue;
					case '\t':
						p.pending = char;
						p.indent += '\t';
						p.indent_len += 4;
						continue;
					case '\n':
						if (p.tokens[p.len] === LIST_ITEM && p.token === LINE_BREAK) {
							end_token(p);
							clear_root_pending(p);
							p.pending = char;
							continue;
						}
						end_tokens_to_len(p, p.blockquote_idx);
						clear_root_pending(p);
						p.blockquote_idx = 0;
						p.fence_start = 0;
						p.pending = char;
						continue;
					case '#':
						switch (char) {
							case '#':
								if (p.pending.length < 6) {
									p.pending = pending_with_char;
									continue;
								}
								break;
							case ' ':
								end_tokens_to_indent(p, p.indent_len);
								add_token(p, level_to_heading(p.pending.length));
								clear_root_pending(p);
								continue;
						}
						break;
					case '>': {
						const next_blockquote_idx = idx_of_token(p, BLOCKQUOTE, p.blockquote_idx + 1);

						if (next_blockquote_idx === -1) {
							end_tokens_to_len(p, p.blockquote_idx);
							p.blockquote_idx += 1;
							p.fence_start = 0;
							add_token(p, BLOCKQUOTE);
						} else {
							p.blockquote_idx = next_blockquote_idx;
						}

						clear_root_pending(p);
						p.pending = char;
						continue;
					}
					case '-':
					case '*':
					case '_':
						if (p.hr_chars === 0) {
							p.hr_chars = 1;
							p.hr_char = p.pending;
						}

						if (p.hr_chars > 0) {
							switch (char) {
								case p.hr_char:
									p.hr_chars += 1;
									p.pending = pending_with_char;
									continue;
								case ' ':
									p.pending = pending_with_char;
									continue;
								case '\n':
									if (p.hr_chars < 3) break;
									end_tokens_to_indent(p, p.indent_len);
									p.renderer.add_token(p.renderer.data, RULE);
									p.renderer.end_token(p.renderer.data);
									clear_root_pending(p);
									p.hr_chars = 0;
									continue;
							}

							p.hr_chars = 0;
						}

						if ('_' !== p.pending[0] && ' ' === p.pending[1]) {
							continue_or_add_list(p, LIST_UNORDERED);
							add_list_item(p, 2);
							parser_write(p, pending_with_char.slice(2));
							continue;
						}

						break;
					case '`':
						if (p.pending.length < 3) {
							if ('`' === char) {
								p.pending = pending_with_char;
								p.fence_start = pending_with_char.length;
								continue;
							}
							p.fence_start = 0;
							break;
						}

						switch (char) {
							case '`':
								if (p.pending.length === p.fence_start) {
									p.pending = pending_with_char;
									p.fence_start = pending_with_char.length;
								} else {
									add_token(p, PARAGRAPH);
									clear_root_pending(p);
									p.fence_start = 0;
									parser_write(p, pending_with_char);
								}
								continue;
							case '\n': {
								end_tokens_to_indent(p, p.indent_len);

								add_token(p, CODE_FENCE);
								if (p.pending.length > p.fence_start) {
									p.renderer.set_attr(p.renderer.data, LANG, p.pending.slice(p.fence_start));
								}
								clear_root_pending(p);
								p.token = NEWLINE;
								continue;
							}
							default:
								p.pending = pending_with_char;
								continue;
						}
					case '+':
						if (' ' !== char) break;

						continue_or_add_list(p, LIST_UNORDERED);
						add_list_item(p, 2);
						continue;
					case '0':
					case '1':
					case '2':
					case '3':
					case '4':
					case '5':
					case '6':
					case '7':
					case '8':
					case '9':
						if ('.' === p.pending[p.pending.length - 1]) {
							if (' ' !== char) break;

							if (continue_or_add_list(p, LIST_ORDERED) && p.pending !== '1.') {
								p.renderer.set_attr(p.renderer.data, START, p.pending.slice(0, -1));
							}
							add_list_item(p, p.pending.length + 1);
							continue;
						} else {
							const char_code = char.charCodeAt(0);
							if (46 === char_code || is_digit(char_code)) {
								p.pending = pending_with_char;
								continue;
							}
						}
						break;
					case '|':
						end_tokens_to_len(p, p.blockquote_idx);

						add_token(p, TABLE);
						add_token(p, TABLE_ROW);

						p.pending = '';
						parser_write(p, char);

						continue;
				}

				let to_write = pending_with_char;

				if (p.token === LINE_BREAK) {
					p.text = '';
					p.token = p.tokens[p.len];
					p.renderer.add_token(p.renderer.data, LINE_BREAK);
					p.renderer.end_token(p.renderer.data);
				} else if (p.indent_len >= 4) {
					let code_start = 0;
					for (; code_start < 4; code_start += 1) {
						if (p.indent[code_start] === '\t') {
							code_start = code_start + 1;
							break;
						}
					}
					to_write = p.indent.slice(code_start) + pending_with_char;
					add_token(p, CODE_BLOCK);
				} else {
					add_token(p, PARAGRAPH);
				}

				clear_root_pending(p);
				parser_write(p, to_write);
				continue;
			case TABLE:
				if (p.table_state === 1) {
					switch (char) {
						case '-':
						case ' ':
						case '|':
						case ':':
							p.pending = pending_with_char;
							continue;
						case '\n':
							p.table_state = 2;
							p.pending = '';
							continue;
						default:
							end_token(p);
							p.table_state = 0;
							break;
					}
				} else {
					switch (p.pending) {
						case '|':
							add_token(p, TABLE_ROW);
							p.pending = '';
							parser_write(p, char);
							continue;
						case '\n':
							end_token(p);
							p.pending = '';
							p.table_state = 0;
							parser_write(p, char);
							continue;
					}
				}
				break;
			case TABLE_ROW:
				switch (p.pending) {
					case '':
						break;
					case '|':
						add_token(p, TABLE_CELL);
						end_token(p);
						p.pending = '';
						parser_write(p, char);
						continue;
					case '\n':
						end_token(p);
						p.table_state = Math.min(p.table_state + 1, 2);
						p.pending = '';
						parser_write(p, char);
						continue;
					default:
						add_token(p, TABLE_CELL);
						parser_write(p, char);
						continue;
				}
				break;
			case TABLE_CELL:
				if (p.pending === '|') {
					add_text(p);
					end_token(p);
					p.pending = '';
					parser_write(p, char);
					continue;
				}
				break;
			case CODE_BLOCK:
				if (p.pending[0] === '\n') {
					const iw = indent_width(pending_with_char.slice(1));
					if (iw >= 4) {
						p.text += '\n';
						p.pending = '';
						continue;
					}
					if (iw === 0) {
						p.pending = pending_with_char;
						continue;
					}
				}
				if (p.pending.length !== 0) {
					add_text(p);
					end_token(p);
					p.pending = char;
				} else {
					p.text += char;
				}
				continue;
			case CODE_FENCE:
				switch (char) {
					case '`':
						p.pending = pending_with_char;
						continue;
					case '\n':
						if (pending_with_char.length === p.fence_start + p.fence_end + 1) {
							add_text(p);
							end_token(p);
							p.pending = '';
							p.fence_start = 0;
							p.fence_end = 0;
							p.token = NEWLINE;
							continue;
						}
						p.token = NEWLINE;
						break;
					case ' ':
						if (p.pending[0] === '\n') {
							p.pending = pending_with_char;
							p.fence_end += 1;
							continue;
						}
						break;
				}
				p.text += p.pending;
				p.pending = char;
				p.fence_end = 1;
				continue;
			case CODE_INLINE:
				switch (char) {
					case '`':
						if (pending_with_char.length === p.fence_start + Number(p.pending[0] === ' ')) {
							add_text(p);
							end_token(p);
							p.pending = '';
							p.fence_start = 0;
						} else {
							p.pending = pending_with_char;
						}
						continue;
					case '\n':
						p.text += p.pending;
						p.pending = '';
						add_text(p);
						end_token(p);
						parser_write(p, '\n');
						continue;
					case ' ':
						p.text += p.pending;
						p.pending = char;
						continue;
					default:
						p.text += pending_with_char;
						p.pending = '';
						continue;
				}
			case MAYBE_TASK:
				switch (p.pending.length) {
					case 0:
						if ('[' !== char) break;
						p.pending = pending_with_char;
						continue;
					case 1:
						if (' ' !== char && 'x' !== char) break;
						p.pending = pending_with_char;
						continue;
					case 2:
						if (']' !== char) break;
						p.pending = pending_with_char;
						continue;
					case 3:
						if (' ' !== char) break;
						p.renderer.add_token(p.renderer.data, CHECKBOX);
						if ('x' === p.pending[1]) {
							p.renderer.set_attr(p.renderer.data, CHECKED, '');
						}
						p.renderer.end_token(p.renderer.data);
						p.pending = ' ';
						continue;
				}

				p.token = p.tokens[p.len];
				p.pending = '';
				parser_write(p, pending_with_char);
				continue;
			case STRONG_AST:
			case STRONG_UND: {
				let symbol = '*';
				let italic = ITALIC_AST;
				if (p.token === STRONG_UND) {
					symbol = '_';
					italic = ITALIC_UND;
				}

				if (symbol === p.pending) {
					add_text(p);
					if (symbol === char) {
						end_token(p);
						p.pending = '';
						continue;
					}
					add_token(p, italic);
					p.pending = char;
					continue;
				}

				break;
			}
			case ITALIC_AST:
			case ITALIC_UND: {
				let symbol = '*';
				let strong = STRONG_AST;
				if (p.token === ITALIC_UND) {
					symbol = '_';
					strong = STRONG_UND;
				}

				switch (p.pending) {
					case symbol:
						if (symbol === char) {
							if (p.tokens[p.len - 1] === strong) {
								p.pending = pending_with_char;
							} else {
								add_text(p);
								add_token(p, strong);
								p.pending = '';
							}
						} else {
							add_text(p);
							end_token(p);
							p.pending = char;
						}
						continue;
					case symbol + symbol:
						const italic = p.token;
						add_text(p);
						end_token(p);
						end_token(p);
						if (symbol !== char) {
							add_token(p, italic);
							p.pending = char;
						} else {
							p.pending = '';
						}
						continue;
				}
				break;
			}
			case STRIKE:
				if ('~~' === pending_with_char) {
					add_text(p);
					end_token(p);
					p.pending = '';
					continue;
				}
				break;
			case MAYBE_EQ_BLOCK:
				if (char === '\n') {
					add_text(p);
					add_token(p, EQUATION_BLOCK_DOLLAR);
					p.pending = '';
				} else {
					p.token = p.tokens[p.len];
					p.text += '$$';
					p.pending = '';
					parser_write(p, char);
				}
				continue;
			case EQUATION_BLOCK_DOLLAR:
				if ('$$' === pending_with_char) {
					add_text(p);
					end_token(p);
					p.pending = '';
					continue;
				}
				break;
			case EQUATION_BLOCK_BRACKET:
				if ('\\]' === pending_with_char) {
					add_text(p);
					end_token(p);
					p.pending = '';
					continue;
				}
				if (p.pending[0] === '\\' && char !== ']') {
					p.text += pending_with_char;
					p.pending = '';
					continue;
				}
				break;
			case EQUATION_INLINE:
				// \\ double backslash — preserve as LaTeX line break
				if (p.pending[0] === '\\' && char === '\\') {
					p.text += '\\\\';
					p.pending = '';
					continue;
				}
				// escaped \$ — preserve backslash for KaTeX
				if (p.pending[0] === '\\' && char === '$') {
					p.text += '\\$';
					p.pending = '';
					continue;
				}
				// explicit escape passthrough: preserve \X for KaTeX
				if (p.pending[0] === '\\' && char !== ')' && char !== '(' && char !== '$') {
					p.text += pending_with_char;
					p.pending = '';
					continue;
				}
				// close only on matching delimiter
				if (p.eq_open === 1 && '$' === p.pending[0]) {
					add_text(p);
					end_token(p);
					p.eq_open = 0;
					p.pending = char;
					continue;
				}
				if (p.eq_open === 2 && '\\)' === pending_with_char) {
					add_text(p);
					end_token(p);
					p.eq_open = 0;
					p.pending = '';
					continue;
				}
				// $ as literal content inside \(…\)
				if (p.eq_open === 2 && '$' === p.pending[0]) {
					p.text += '$';
					p.pending = char;
					continue;
				}
				break;
			case MAYBE_URL:
				if ('http://' === pending_with_char || 'https://' === pending_with_char) {
					add_text(p);
					add_token(p, RAW_URL);
					p.pending = pending_with_char;
					p.text = pending_with_char;
				} else if ('http:/'[p.pending.length] === char || 'https:/'[p.pending.length] === char) {
					p.pending = pending_with_char;
				} else {
					p.token = p.tokens[p.len];
					parser_write(p, char);
				}
				continue;
			case MAYBE_LINK:
				if (']' === p.pending) {
					if ('(' === char) {
						const saved = p.maybe_link_text;
						p.token = p.tokens[p.len];
						p.pending = '';
						add_token(p, LINK);
						parser_write(p, saved);
						p.text += p.pending;
						p.pending = '';
						add_text(p);
						p.pending = '](';
					} else {
						p.text = '[';
						add_text(p);
						p.token = p.tokens[p.len];
						p.pending = '';
						parser_write(p, p.maybe_link_text);
						p.text += p.pending;
						p.pending = '';
						add_text(p);
						p.text = ']';
						add_text(p);
						parser_write(p, char);
					}
					continue;
				}
				if (']' === char) {
					p.maybe_link_text += p.pending;
					p.pending = ']';
					continue;
				}
				if ('\n' === char) {
					p.token = p.tokens[p.len];
					p.pending = '';
					p.text = '[';
					add_text(p);
					parser_write(p, p.maybe_link_text + p.text);
					p.text += p.pending;
					p.pending = '';
					add_text(p);
					parser_write(p, char);
					continue;
				}
				p.maybe_link_text += p.pending;
				p.pending = char;
				continue;
			case LINK:
			case IMAGE:
				if (']' === p.pending) {
					add_text(p);
					if ('(' === char) {
						p.pending = pending_with_char;
					} else {
						end_token(p);
						p.pending = char;
					}
					continue;
				}
				if (']' === p.pending[0] && '(' === p.pending[1]) {
					if (')' === char) {
						const type = p.token === LINK ? HREF : SRC;
						const url = p.pending.slice(2);
						p.renderer.set_attr(p.renderer.data, type, url);
						end_token(p);
						p.pending = '';
					} else {
						p.pending += char;
					}
					continue;
				}
				break;
			case RAW_URL:
				if (' ' === char || '\n' === char || '\\' === char) {
					p.renderer.set_attr(p.renderer.data, HREF, p.pending);
					add_text(p);
					end_token(p);
					p.pending = char;
				} else {
					p.text += char;
					p.pending = pending_with_char;
				}
				continue;
			case MAYBE_BR:
				if (pending_with_char.startsWith('  ') && char === '\n') {
					add_text(p);
					p.token = p.tokens[p.len];
					p.renderer.add_token(p.renderer.data, LINE_BREAK);
					p.renderer.end_token(p.renderer.data);
					p.pending = '';
					continue;
				}
				if (pending_with_char.startsWith('\\')) {
					if (char === '\n') {
						add_text(p);
						p.token = p.tokens[p.len];
						p.renderer.add_token(p.renderer.data, LINE_BREAK);
						p.renderer.end_token(p.renderer.data);
						p.pending = '';
						continue;
					}
				}
				if (char === '>') {
					add_text(p);
					p.token = p.tokens[p.len];
					p.renderer.add_token(p.renderer.data, LINE_BREAK);
					p.renderer.end_token(p.renderer.data);
					p.pending = '';
					continue;
				}
				p.token = p.tokens[p.len];
				p.text += '<';
				p.pending = p.pending.slice(1);
				parser_write(p, char);
				continue;
		}

		switch (p.pending[0]) {
			case '\\':
				if (p.token === IMAGE || is_block_eq(p.token) || p.token === EQUATION_INLINE) break;

				switch (char) {
					case '(':
						add_text(p);
						add_token(p, EQUATION_INLINE);
						p.eq_open = 2;
						p.pending = '';
						continue;
					case '[':
						add_text(p);
						add_token(p, EQUATION_BLOCK_BRACKET);
						p.pending = '';
						continue;
					case '\n':
						p.pending = char;
						continue;
					default:
						let charcode = char.charCodeAt(0);
						p.pending = '';
						p.text +=
							is_digit(charcode) ||
							(charcode >= 65 && charcode <= 90) ||
							(charcode >= 97 && charcode <= 122)
								? pending_with_char
								: char;
						continue;
				}
			case '\n':
				switch (p.token) {
					case IMAGE:
					case EQUATION_BLOCK:
					case EQUATION_BLOCK_DOLLAR:
					case EQUATION_BLOCK_BRACKET:
					case EQUATION_INLINE:
						break;
					case ITALIC_AST:
					case ITALIC_UND:
					case STRONG_AST:
					case STRONG_UND:
					case STRIKE:
					case CODE_INLINE:
						add_text(p);
						end_token(p);
						p.pending = '';
						parser_write(p, pending_with_char);
						continue;
					case HEADING_1:
					case HEADING_2:
					case HEADING_3:
					case HEADING_4:
					case HEADING_5:
					case HEADING_6:
						add_text(p);
						end_tokens_to_len(p, p.blockquote_idx);
						p.blockquote_idx = 0;
						p.pending = char;
						continue;
					default:
						add_text(p);
						p.pending = char;
						p.token = LINE_BREAK;
						p.blockquote_idx = 0;
						continue;
				}
				break;
			case '<':
				if (p.token !== IMAGE && !is_block_eq(p.token) && p.token !== EQUATION_INLINE) {
					add_text(p);
					p.pending = pending_with_char;
					p.token = MAYBE_BR;
					continue;
				}
				break;
			case '`':
				if (p.token === IMAGE || is_block_eq(p.token) || p.token === EQUATION_INLINE) break;

				if ('`' === char) {
					p.fence_start += 1;
					p.pending = pending_with_char;
				} else {
					p.fence_start += 1;
					add_text(p);
					add_token(p, CODE_INLINE);
					p.text = ' ' === char || '\n' === char ? '' : char;
					p.pending = '';
				}
				continue;
			case '_':
			case '*': {
				if (
					p.token === IMAGE ||
					is_block_eq(p.token) ||
					p.token === EQUATION_INLINE ||
					p.token === STRONG_AST
				)
					break;

				let italic = ITALIC_AST;
				let strong = STRONG_AST;
				const symbol = p.pending[0];
				if ('_' === symbol) {
					italic = ITALIC_UND;
					strong = STRONG_UND;
				}

				// _ between two alphanumeric chars is not emphasis (e.g., foo_bar, a_b)
				if (
					symbol === '_' &&
					p.text.length > 0 &&
					is_alphanumeric(p.text[p.text.length - 1]) &&
					is_alphanumeric(char)
				) {
					p.text += p.pending;
					p.pending = char;
					continue;
				}

				if (p.pending.length === 1) {
					if (symbol === char) {
						p.pending = pending_with_char;
						continue;
					}
					if (' ' !== char && '\n' !== char) {
						add_text(p);
						add_token(p, italic);
						p.pending = char;
						continue;
					}
				} else {
					if (symbol === char) {
						add_text(p);
						add_token(p, strong);
						add_token(p, italic);
						p.pending = '';
						continue;
					}
					if (' ' !== char && '\n' !== char) {
						add_text(p);
						add_token(p, strong);
						p.pending = char;
						continue;
					}
				}

				break;
			}
			case '~':
				if (
					p.token !== IMAGE &&
					p.token !== STRIKE &&
					!is_block_eq(p.token) &&
					p.token !== EQUATION_INLINE
				) {
					if ('~' === p.pending) {
						if ('~' === char) {
							p.pending = pending_with_char;
							continue;
						}
					} else {
						if (' ' !== char && '\n' !== char) {
							add_text(p);
							add_token(p, STRIKE);
							p.pending = char;
							continue;
						}
					}
				}
				break;
			case '$':
				if (
					p.token !== IMAGE &&
					p.token !== STRIKE &&
					!is_block_eq(p.token) &&
					p.token !== EQUATION_INLINE &&
					'$' === p.pending
				) {
					if ('$' === char) {
						p.token = MAYBE_EQ_BLOCK;
						p.pending = pending_with_char;
						continue;
					} else if (is_delimeter_or_number(char.charCodeAt(0))) {
						break;
					} else {
						add_text(p);
						add_token(p, EQUATION_INLINE);
						p.eq_open = 1;
						p.pending = char;
						continue;
					}
				}
				break;
			case '[':
				if (
					p.token !== IMAGE &&
					p.token !== LINK &&
					!is_block_eq(p.token) &&
					p.token !== EQUATION_INLINE &&
					']' !== char
				) {
					add_text(p);
					p.token = MAYBE_LINK;
					p.maybe_link_text = '';
					p.pending = char;
					continue;
				}
				break;
			case '!':
				if (
					p.token !== IMAGE &&
					!is_block_eq(p.token) &&
					p.token !== EQUATION_INLINE &&
					'[' === char
				) {
					add_text(p);
					add_token(p, IMAGE);
					p.pending = '';
					continue;
				}
				break;
			case ' ':
				if (p.pending.length === 1 && ' ' === char) {
					continue;
				}
				break;
		}

		if (
			p.token !== IMAGE &&
			p.token !== LINK &&
			!is_block_eq(p.token) &&
			p.token !== EQUATION_INLINE &&
			'h' === char &&
			(' ' === p.pending || '' === p.pending)
		) {
			p.text += p.pending;
			p.pending = char;

			p.token = MAYBE_URL;
			continue;
		}

		p.text += p.pending;
		p.pending = char;
	}

	add_text(p);
}
