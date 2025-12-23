import { describe, test, expect } from 'vitest';
import { parse } from '../parser';
import { TokenType } from '../tokens';
import type { ParagraphToken, TextToken } from '../tokens';
describe('Manual test', () => {
	test('1|2', () => {
		const markdown = '1|2';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		expect(para.children.length).toBe(1);
		expect(para.children[0].type).toBe(TokenType.Text);
		expect((para.children[0] as TextToken).content).toBe('1|2');
	});
});
