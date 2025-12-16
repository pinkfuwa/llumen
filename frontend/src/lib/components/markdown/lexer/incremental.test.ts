import { describe, test, expect } from 'vitest';
import { parseIncremental, type IncrementalState } from './index';
import { TokenType } from './tokens';

describe('Incremental Parsing', () => {
	test('first parse with no state creates initial state', async () => {
		const source = '# Hello\n\nWorld';
		const result = await parseIncremental(source, null);

		expect(result.result.tokens).toHaveLength(2);
		expect(result.state.prevSource).toBe(source);
		expect(result.state.newContentStart).toBe(source.length);
		expect(result.state.prevResult).toBe(result.result);
	});

	test('extending with whitespace only reuses previous result', async () => {
		const source1 = '# Hello\n\nWorld';
		const first = await parseIncremental(source1, null);

		const source2 = source1 + '   \n  ';
		const second = await parseIncremental(source2, first.state);

		expect(second.result.tokens).toHaveLength(2);
		expect(second.result).toBe(first.state.prevResult);
	});

	test('non-extending source triggers full reparse', async () => {
		const source1 = '# Hello\n\nWorld';
		const first = await parseIncremental(source1, null);

		const source2 = '# Different';
		const second = await parseIncremental(source2, first.state);

		expect(second.result.tokens).toHaveLength(1);
		expect(second.result.tokens[0].type).toBe(TokenType.Heading);
	});

	test('incremental parse preserves completed regions', async () => {
		const source1 = '# Hello\n\nFirst paragraph.';
		const first = await parseIncremental(source1, null);

		const source2 = source1 + '\n\nSecond paragraph.';
		const second = await parseIncremental(source2, first.state);

		expect(second.result.tokens).toHaveLength(3);
		expect(second.result.tokens[0].type).toBe(TokenType.Heading);
		expect(second.result.tokens[1].type).toBe(TokenType.Paragraph);
		expect(second.result.tokens[2].type).toBe(TokenType.Paragraph);
	});

	test('incremental parse with code block region', async () => {
		const source1 = '```js\nconst x = 1;\n```\n\n';
		const first = await parseIncremental(source1, null);

		const source2 = source1 + 'Next paragraph.';
		const second = await parseIncremental(source2, first.state);

		expect(second.result.tokens).toHaveLength(2);
		expect(second.result.tokens[0].type).toBe(TokenType.CodeBlock);
		expect(second.result.tokens[1].type).toBe(TokenType.Paragraph);
		expect(second.result.regions.some((r) => r.type === 'codeblock')).toBe(true);
	});

	test('incremental parse with table region', async () => {
		const source1 = '| A | B |\n|---|---|\n| 1 | 2 |\n\n';
		const first = await parseIncremental(source1, null);

		const source2 = source1 + 'After table.';
		const second = await parseIncremental(source2, first.state);

		expect(second.result.tokens).toHaveLength(2);
		expect(second.result.tokens[0].type).toBe(TokenType.Table);
		expect(second.result.tokens[1].type).toBe(TokenType.Paragraph);
		expect(second.result.regions.some((r) => r.type === 'table')).toBe(true);
	});

	test('incremental parse with list region', async () => {
		const source1 = '- Item 1\n- Item 2\n\n';
		const first = await parseIncremental(source1, null);

		const source2 = source1 + 'After list.';
		const second = await parseIncremental(source2, first.state);

		expect(second.result.tokens).toHaveLength(2);
		expect(second.result.tokens[0].type).toBe(TokenType.UnorderedList);
		expect(second.result.tokens[1].type).toBe(TokenType.Paragraph);
		expect(second.result.regions.some((r) => r.type === 'list')).toBe(true);
	});

	test('incremental parse with blockquote region', async () => {
		const source1 = '> Quote\n> More quote\n\n';
		const first = await parseIncremental(source1, null);

		const source2 = source1 + 'After quote.';
		const second = await parseIncremental(source2, first.state);

		expect(second.result.tokens).toHaveLength(2);
		expect(second.result.tokens[0].type).toBe(TokenType.Blockquote);
		expect(second.result.tokens[1].type).toBe(TokenType.Paragraph);
		expect(second.result.regions.some((r) => r.type === 'blockquote')).toBe(true);
	});

	test('incremental parse with paragraph region', async () => {
		const source1 = 'First paragraph.\n\n';
		const first = await parseIncremental(source1, null);

		const source2 = source1 + 'Second paragraph.';
		const second = await parseIncremental(source2, first.state);

		expect(second.result.tokens).toHaveLength(2);
		expect(second.result.tokens[0].type).toBe(TokenType.Paragraph);
		expect(second.result.tokens[1].type).toBe(TokenType.Paragraph);
		expect(second.result.regions.some((r) => r.type === 'paragraph')).toBe(true);
	});

	test('incremental parse adjusts token positions correctly', async () => {
		const source1 = '# Heading\n\nParagraph one.';
		const first = await parseIncremental(source1, null);

		const source2 = source1 + '\n\n## Second heading';
		const second = await parseIncremental(source2, first.state);

		expect(second.result.tokens).toHaveLength(3);
		const lastToken = second.result.tokens[2];
		expect(lastToken.type).toBe(TokenType.Heading);
		expect(lastToken.start).toBeGreaterThan(source1.length);
		expect(lastToken.end).toBeLessThanOrEqual(source2.length);
	});

	test('incremental parse handles multiple extensions', async () => {
		const source1 = '# Title\n\n';
		const first = await parseIncremental(source1, null);

		const source2 = source1 + 'Para 1.\n\n';
		const second = await parseIncremental(source2, first.state);

		const source3 = source2 + 'Para 2.\n\n';
		const third = await parseIncremental(source3, second.state);

		const source4 = source3 + '```\ncode\n```';
		const fourth = await parseIncremental(source4, third.state);

		expect(fourth.result.tokens).toHaveLength(4);
		expect(fourth.result.tokens[0].type).toBe(TokenType.Heading);
		expect(fourth.result.tokens[1].type).toBe(TokenType.Paragraph);
		expect(fourth.result.tokens[2].type).toBe(TokenType.Paragraph);
		expect(fourth.result.tokens[3].type).toBe(TokenType.CodeBlock);
	});

	test('incremental parse with mixed content', async () => {
		const source1 = '# Header\n\nParagraph.\n\n';
		const first = await parseIncremental(source1, null);

		const source2 = source1 + '- List item\n\n';
		const second = await parseIncremental(source2, first.state);

		const source3 = source2 + '> Quote\n\n';
		const third = await parseIncremental(source3, second.state);

		expect(third.result.tokens).toHaveLength(4);
		expect(third.result.tokens[0].type).toBe(TokenType.Heading);
		expect(third.result.tokens[1].type).toBe(TokenType.Paragraph);
		expect(third.result.tokens[2].type).toBe(TokenType.UnorderedList);
		expect(third.result.tokens[3].type).toBe(TokenType.Blockquote);
	});

	test('region boundaries are preserved across incremental parses', async () => {
		const source1 = '```js\ncode\n```\n\n';
		const first = await parseIncremental(source1, null);
		const region1Count = first.result.regions.length;

		const source2 = source1 + '| A |\n|---|\n| B |\n\n';
		const second = await parseIncremental(source2, first.state);

		expect(second.result.regions.length).toBeGreaterThan(region1Count);
		expect(second.result.regions.some((r) => r.type === 'codeblock')).toBe(true);
		expect(second.result.regions.some((r) => r.type === 'table')).toBe(true);
	});

	test('incomplete regions at end trigger reparse', async () => {
		const source1 = '# Header\n\nComplete para.\n\n';
		const first = await parseIncremental(source1, null);

		const source2 = source1 + 'Incomplete para';
		const second = await parseIncremental(source2, first.state);

		expect(second.result.tokens).toHaveLength(3);
		// Last token should include the incomplete paragraph
		expect(second.result.tokens[2].type).toBe(TokenType.Paragraph);
	});

	test('handles empty initial state', async () => {
		const result = await parseIncremental('', null);
		expect(result.result.tokens).toHaveLength(0);
		expect(result.state.prevSource).toBe('');
	});

	test('consecutive incremental parses maintain consistency', async () => {
		let state: IncrementalState | null = null;
		let source = '';

		// Add heading
		source += '# Title\n\n';
		const r1 = await parseIncremental(source, state);
		state = r1.state;

		// Add paragraph
		source += 'First paragraph.\n\n';
		const r2 = await parseIncremental(source, state);
		state = r2.state;

		// Add list
		source += '- Item 1\n- Item 2\n\n';
		const r3 = await parseIncremental(source, state);
		state = r3.state;

		// Add code block
		source += '```python\nprint("hi")\n```';
		const r4 = await parseIncremental(source, state);

		expect(r4.result.tokens).toHaveLength(4);
		expect(r4.result.tokens[0].type).toBe(TokenType.Heading);
		expect(r4.result.tokens[1].type).toBe(TokenType.Paragraph);
		expect(r4.result.tokens[2].type).toBe(TokenType.UnorderedList);
		expect(r4.result.tokens[3].type).toBe(TokenType.CodeBlock);
	});
});
