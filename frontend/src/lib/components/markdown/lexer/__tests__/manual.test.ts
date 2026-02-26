import { describe, test, expect } from 'vitest';
import { parse } from '../index';
import { TokenType } from '../tokens';
import type { ParagraphToken, UnorderedListToken } from '../tokens';
describe('Manual test', () => {
	test('1|2', () => {
		const markdown = '1|2';
		const result = parse(markdown);
		const para = result.tokens[0] as ParagraphToken;
		expect(para.children.length).toBe(1);
		expect(para.children[0].type).toBe(TokenType.Text);
	});

	test('nested list with inline formatting (indented)', () => {
		const markdown = `- \`_start\` is **not** in the C library (\`libc.so\`)
- It's in the **C runtime startup object files**:
  - \`crt1.o\` — for dynamically linked executables
  - \`rcrt1.o\` — for statically linked executables
  - \`Scrt1.o\` — for position-independent executables (PIE)
`;
		const result = parse(markdown);
		expect(result.tokens.length).toBe(2);
		const list = result.tokens[0] as UnorderedListToken;
		expect(list.type).toBe(TokenType.UnorderedList);
		expect(list.children.length).toBe(2);
		const nestedList = result.tokens[1] as UnorderedListToken;
		expect(nestedList.type).toBe(TokenType.UnorderedList);
		expect(nestedList.children.length).toBe(3);
	});
});
