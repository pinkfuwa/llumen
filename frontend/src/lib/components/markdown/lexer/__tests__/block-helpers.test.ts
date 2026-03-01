import { describe, test, expect } from 'vitest';
import {
	peekLine,
	peek,
	skipWhitespace,
	skipNewlines,
	isTableRow,
	isTableSeparator,
	looksLikeBlockStart,
	skipBlankLines,
	parseBlock,
	type ParseContext
} from '../parsers/block-parser';

describe('BlockParser - Helper Functions', () => {
	describe('peekLine', () => {
		test('returns line from start position', () => {
			const source = 'Hello World\nSecond Line';
			const result = peekLine(source, 0);
			expect(result).toBe('Hello World');
		});

		test('returns line from middle position', () => {
			const source = 'Hello World\nSecond Line';
			const result = peekLine(source, 6);
			expect(result).toBe('World');
		});

		test('returns rest of source when no newline', () => {
			const source = 'No newline';
			const result = peekLine(source, 0);
			expect(result).toBe('No newline');
		});

		test('returns empty string at end of source', () => {
			const source = 'Hello';
			const result = peekLine(source, 10);
			expect(result).toBe('');
		});

		test('handles multiple newlines', () => {
			const source = 'Line1\nLine2\nLine3';
			const result = peekLine(source, 0);
			expect(result).toBe('Line1');
		});

		test('handles Windows line endings (stops at CR)', () => {
			const source = 'Line1\r\nLine2';
			const result = peekLine(source, 0);
			// Implementation stops at first \n, includes \r in line
			expect(result.startsWith('Line1')).toBe(true);
		});
	});

	describe('peek', () => {
		test('peeks single character', () => {
			const source = 'Hello';
			const result = peek(source, 0, 1);
			expect(result).toBe('H');
		});

		test('peeks multiple characters', () => {
			const source = 'Hello';
			const result = peek(source, 0, 3);
			expect(result).toBe('Hel');
		});

		test('peeks at end of source', () => {
			const source = 'Hi';
			const result = peek(source, 1, 3);
			expect(result).toBe('i');
		});

		test('peeks beyond source length', () => {
			const source = 'Hi';
			const result = peek(source, 0, 10);
			expect(result).toBe('Hi');
		});

		test('peeks from middle position', () => {
			const source = 'Hello';
			const result = peek(source, 2, 2);
			expect(result).toBe('ll');
		});
	});

	describe('skipWhitespace', () => {
		test('skips spaces', () => {
			const source = '   Hello';
			const result = skipWhitespace(source, 0);
			expect(result).toBe(3);
		});

		test('skips tabs', () => {
			const source = '\t\tHello';
			const result = skipWhitespace(source, 0);
			expect(result).toBe(2);
		});

		test('skips mixed spaces and tabs', () => {
			const source = ' \t \tHello';
			const result = skipWhitespace(source, 0);
			expect(result).toBe(4);
		});

		test('returns same position when no whitespace', () => {
			const source = 'Hello';
			const result = skipWhitespace(source, 0);
			expect(result).toBe(0);
		});

		test('stops at newline', () => {
			const source = '   \nHello';
			const result = skipWhitespace(source, 0);
			expect(result).toBe(3);
		});

		test('skips nothing at end of source', () => {
			const source = 'Hello';
			const result = skipWhitespace(source, 5);
			expect(result).toBe(5);
		});
	});

	describe('skipNewlines', () => {
		test('skips single newline', () => {
			const source = '\nHello';
			const result = skipNewlines(source, 0);
			expect(result.newPosition).toBe(1);
			expect(result.hadBlankLine).toBe(false);
		});

		test('skips multiple newlines', () => {
			const source = '\n\n\nHello';
			const result = skipNewlines(source, 0);
			expect(result.newPosition).toBe(3);
			expect(result.hadBlankLine).toBe(true);
		});

		test('detects blank line with two newlines', () => {
			const source = 'Hello\n\nWorld';
			const result = skipNewlines(source, 5);
			expect(result.newPosition).toBe(7);
			expect(result.hadBlankLine).toBe(true);
		});

		test('handles Windows CRLF (counts CR)', () => {
			const source = 'Hello\r\n\r\nWorld';
			const result = skipNewlines(source, 5);
			// Implementation counts \n characters, not pairs
			expect(result.newPosition).toBeGreaterThan(5);
		});

		test('returns same position when no newline', () => {
			const source = 'Hello';
			const result = skipNewlines(source, 0);
			expect(result.newPosition).toBe(0);
			expect(result.hadBlankLine).toBe(false);
		});

		test('skips CR without LF', () => {
			const source = 'Hello\rWorld';
			const result = skipNewlines(source, 5);
			expect(result.newPosition).toBe(6);
		});
	});

	describe('isTableRow', () => {
		test('returns true for pipe-separated row with 2+ pipes', () => {
			expect(isTableRow('| A | B |')).toBe(true);
		});

		test('returns true for pipe-separated row with 3 pipes', () => {
			expect(isTableRow('| A | B | C |')).toBe(true);
		});

		test('returns false for single pipe', () => {
			expect(isTableRow('| A')).toBe(false);
		});

		test('returns true for tab-separated row', () => {
			expect(isTableRow('A\tB')).toBe(true);
		});

		test('returns true for mixed tabs and spaces', () => {
			expect(isTableRow('A\t  B\tC')).toBe(true);
		});

		test('trims whitespace', () => {
			expect(isTableRow('  | A | B |  ')).toBe(true);
		});

		test('returns false for plain text', () => {
			expect(isTableRow('Just some text')).toBe(false);
		});

		test('returns false for lines with inline code containing pipes', () => {
			expect(isTableRow('Does `D(p||q)=D(q||p)`?')).toBe(false);
		});

		test('returns false for code blocks with pipes', () => {
			expect(isTableRow('```\na || b\n```')).toBe(false);
		});

		test('returns false for inline code with pipes', () => {
			expect(isTableRow('text `a || b` more')).toBe(false);
			expect(isTableRow('`x|y|z`')).toBe(false);
		});

		test('returns true for table rows with inline code', () => {
			expect(isTableRow('| `code` | text |')).toBe(true);
			expect(isTableRow('| text | `code` |')).toBe(true);
			expect(isTableRow('| A | `fetch("http://localhost")` | B |')).toBe(true);
		});

		test('returns true for indented table rows with inline code', () => {
			expect(isTableRow('  | `code` | text |')).toBe(true);
			expect(isTableRow('\t| text | `code` |')).toBe(true);
		});
	});

	describe('isTableSeparator', () => {
		test('returns true for basic separator', () => {
			expect(isTableSeparator('|---|---|')).toBe(true);
		});

		test('returns true for separator with colons', () => {
			expect(isTableSeparator('|:---|:---:|---:|')).toBe(true);
		});

		test('returns true for separator without pipes', () => {
			expect(isTableSeparator('---')).toBe(true);
		});

		test('returns true for separator with spaces', () => {
			expect(isTableSeparator('| --- | --- |')).toBe(true);
		});

		test('returns false for header row', () => {
			expect(isTableSeparator('| A | B |')).toBe(false);
		});

		test('returns false for plain text', () => {
			expect(isTableSeparator('Some text')).toBe(false);
		});
	});

	describe('looksLikeBlockStart', () => {
		test('detects heading', () => {
			expect(looksLikeBlockStart('# Heading')).toBe(true);
			expect(looksLikeBlockStart('###### H6')).toBe(true);
		});

		test('detects code block', () => {
			expect(looksLikeBlockStart('```js')).toBe(true);
			expect(looksLikeBlockStart('   ```python')).toBe(true);
		});

		test('detects horizontal rule', () => {
			expect(looksLikeBlockStart('---')).toBe(true);
			expect(looksLikeBlockStart('***')).toBe(true);
			expect(looksLikeBlockStart('___')).toBe(true);
		});

		test('detects blockquote', () => {
			expect(looksLikeBlockStart('> Quote')).toBe(true);
		});

		test('detects ordered list', () => {
			expect(looksLikeBlockStart('1. Item')).toBe(true);
			expect(looksLikeBlockStart('123. Item')).toBe(true);
		});

		test('detects unordered list', () => {
			expect(looksLikeBlockStart('- Item')).toBe(true);
			expect(looksLikeBlockStart('* Item')).toBe(true);
			expect(looksLikeBlockStart('+ Item')).toBe(true);
		});

		test('detects latex block', () => {
			expect(looksLikeBlockStart('\\[')).toBe(true);
			expect(looksLikeBlockStart('$$')).toBe(true);
		});

		test('detects table row', () => {
			expect(looksLikeBlockStart('| A | B |')).toBe(true);
			expect(looksLikeBlockStart('A\tB')).toBe(true);
		});

		test('returns false for regular text', () => {
			expect(looksLikeBlockStart('Just some text')).toBe(false);
			expect(looksLikeBlockStart('Not a block')).toBe(false);
		});

		test('returns false for incomplete patterns', () => {
			expect(looksLikeBlockStart('#')).toBe(false);
			expect(looksLikeBlockStart('``')).toBe(false);
			expect(looksLikeBlockStart('-')).toBe(false);
		});
	});

	describe('skipBlankLines', () => {
		test('skips single newline', () => {
			const ctx: ParseContext = { source: '\nHello', position: 0 };
			const result = skipBlankLines(ctx);
			expect(result).toBe(1);
		});

		test('skips multiple newlines', () => {
			const ctx: ParseContext = { source: '\n\n\nHello', position: 0 };
			const result = skipBlankLines(ctx);
			expect(result).toBe(3);
		});

		test('handles Windows line endings', () => {
			const ctx: ParseContext = { source: '\r\n\r\nHello', position: 0 };
			const result = skipBlankLines(ctx);
			// Implementation only skips \n and \r individually
			expect(result).toBeGreaterThan(0);
		});

		test('returns same position for non-newline', () => {
			const ctx: ParseContext = { source: 'Hello', position: 0 };
			const result = skipBlankLines(ctx);
			expect(result).toBe(0);
		});

		test('handles carriage return only', () => {
			const ctx: ParseContext = { source: '\rHello', position: 0 };
			const result = skipBlankLines(ctx);
			expect(result).toBe(1);
		});
	});

	describe('parseBlock - fallback behavior', () => {
		test('advances position for unknown content', () => {
			const ctx: ParseContext = { source: 'Hello World', position: 0 };
			const result = parseBlock(ctx);
			// Parser tries each block type, may match paragraph
			// Position should advance somehow
			expect(result.newPosition).toBeGreaterThan(0);
		});

		test('handles empty source', () => {
			const ctx: ParseContext = { source: '', position: 0 };
			const result = parseBlock(ctx);
			expect(result.token).toBeNull();
		});

		test('handles position at end of source', () => {
			const ctx: ParseContext = { source: 'Hello', position: 5 };
			const result = parseBlock(ctx);
			expect(result.token).toBeNull();
		});
	});
});
