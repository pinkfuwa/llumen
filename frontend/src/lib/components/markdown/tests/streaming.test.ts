import { describe, expect, it } from 'vitest';
import { createAstRenderer, parseSync } from '../parser/renderer';
import { parser, parser_end, parser_write } from '../parser/smd';
import { AstNodeType, type AstNode, type CodeBlockNode } from '../parser/types';

describe('streaming Markdown', () => {
	it('uses UTF-16 source offsets for text containing emoji', () => {
		const source = 'A👋B';
		const nodes = parseSync(source);
		expect(nodes[0].children).toEqual([
			{ type: AstNodeType.Text, content: source, start: 0, end: source.length }
		]);
	});

	it.each([
		'Hello 世界 👋\n\n**bold** and _italic_ with `code`.\n',
		'```typescript\nconst answer = 42;\n```\n\nNext paragraph.\n',
		'> Quote\n> second line\n\n- first\n- second\n',
		'[link](https://example.com) and ![image](https://example.com/a.png)\n',
		'Inline $x + y$ and \\(z\\).\n\n$$\nx^2\n$$\n',
		'| Name | Value |\n| --- | --- |\n| first | **42** |\n',
		'Unfinished **bold and an unfinished `code'
	])('preserves the AST across every split in %j', (source) => {
		const expected = parseSync(source);
		for (let split = 0; split <= source.length; split++) {
			const { renderer, getResult } = createAstRenderer();
			const state = parser(renderer);
			parser_write(state, source.slice(0, split));
			parser_write(state, source.slice(split));
			parser_end(state);
			expect(getResult(), `split at ${split}`).toEqual(expected);
		}
	});

	it('preserves completed nodes while tokens arrive one character at a time', () => {
		const nodes: AstNode[] = [];
		const { renderer, getResult } = createAstRenderer(nodes);
		const state = parser(renderer);
		parser_write(state, 'First paragraph.\n\n`');
		const firstParagraph = nodes[0];
		const firstText = firstParagraph.children?.[0];
		const source = '``js\nconst value = 1;\n```\n\nLast paragraph.\n';
		for (const character of source) {
			parser_write(state, character);
			expect(nodes[0]).toBe(firstParagraph);
			expect(nodes[0].children?.[0]).toBe(firstText);
		}
		parser_end(state);
		expect(getResult()).toBe(nodes);
		expect(nodes).toEqual(parseSync('First paragraph.\n\n`' + source));
	});

	it('updates an open code block in place and closes it before the message ends', () => {
		const nodes: AstNode[] = [];
		const { renderer } = createAstRenderer(nodes);
		const state = parser(renderer);
		parser_write(state, '```typescript\nconst first = 1;\n');
		const code = nodes[0] as CodeBlockNode;
		expect(code.type).toBe(AstNodeType.CodeBlock);
		expect(code.language).toBe('typescript');
		expect(code.closed).toBe(false);
		expect(code.content).toContain('const first = 1;');

		parser_write(state, 'const second = 2;\n');
		expect(nodes[0]).toBe(code);
		expect(code.content).toContain('const second = 2;');
		parser_write(state, '```\n\nMore text');
		expect(nodes[0]).toBe(code);
		expect(code.closed).toBe(true);
		expect(code.content).toBe('const first = 1;\nconst second = 2;');
	});
});
