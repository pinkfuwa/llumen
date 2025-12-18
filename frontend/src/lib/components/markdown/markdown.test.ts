import { describe, test, expect } from 'vitest';
import { parse } from './lexer';
import { TokenType } from './lexer';

describe('Markdown End-to-End Tests', () => {
	test('parses simple text into paragraph with text token', () => {
		const source = 'Hello world';
		const result = parse(source);

		expect(result.tokens).toHaveLength(1);
		expect(result.tokens[0].type).toBe(TokenType.Paragraph);
		expect(result.tokens[0].children).toHaveLength(1);
		expect(result.tokens[0].children![0].type).toBe(TokenType.Text);
	});

	test('parses heading correctly', () => {
		const source = '# Hello World';
		const result = parse(source);

		expect(result.tokens).toHaveLength(1);
		expect(result.tokens[0].type).toBe(TokenType.Heading);
		expect((result.tokens[0] as any).level).toBe(1);
	});

	test('parses multiple headings with different levels', () => {
		const source = '# H1\n\n## H2\n\n### H3';
		const result = parse(source);

		expect(result.tokens).toHaveLength(3);
		expect(result.tokens[0].type).toBe(TokenType.Heading);
		expect((result.tokens[0] as any).level).toBe(1);
		expect(result.tokens[1].type).toBe(TokenType.Heading);
		expect((result.tokens[1] as any).level).toBe(2);
		expect(result.tokens[2].type).toBe(TokenType.Heading);
		expect((result.tokens[2] as any).level).toBe(3);
	});

	test('parses code block with language', () => {
		const source = '```javascript\nconst x = 1;\n```';
		const result = parse(source);

		expect(result.tokens).toHaveLength(1);
		expect(result.tokens[0].type).toBe(TokenType.CodeBlock);
		expect((result.tokens[0] as any).language).toBe('javascript');
		expect((result.tokens[0] as any).content).toBe('const x = 1;');
	});

	test('parses inline formatting (bold, italic, code)', () => {
		const source = 'Text with **bold** and *italic* and `code`';
		const result = parse(source);

		expect(result.tokens).toHaveLength(1);
		expect(result.tokens[0].type).toBe(TokenType.Paragraph);

		const children = result.tokens[0].children || [];
		const hasBold = children.some((t) => t.type === TokenType.Bold);
		const hasItalic = children.some((t) => t.type === TokenType.Italic);
		const hasCode = children.some((t) => t.type === TokenType.InlineCode);

		expect(hasBold).toBe(true);
		expect(hasItalic).toBe(true);
		expect(hasCode).toBe(true);
	});

	test('parses links correctly', () => {
		const source = '[Link text](https://example.com)';
		const result = parse(source);

		expect(result.tokens).toHaveLength(1);
		expect(result.tokens[0].type).toBe(TokenType.Paragraph);

		const children = result.tokens[0].children || [];
		const link = children.find((t) => t.type === TokenType.Link);

		expect(link).toBeDefined();
		expect((link as any).url).toBe('https://example.com');
	});

	test('parses images correctly', () => {
		const source = '![Alt text](https://example.com/image.png)';
		const result = parse(source);

		expect(result.tokens).toHaveLength(1);
		expect(result.tokens[0].type).toBe(TokenType.Paragraph);

		const children = result.tokens[0].children || [];
		const image = children.find((t) => t.type === TokenType.Image);

		expect(image).toBeDefined();
		expect((image as any).url).toBe('https://example.com/image.png');
		expect((image as any).alt).toBe('Alt text');
	});

	test('parses unordered list', () => {
		const source = '- Item 1\n- Item 2\n- Item 3';
		const result = parse(source);

		expect(result.tokens).toHaveLength(1);
		expect(result.tokens[0].type).toBe(TokenType.UnorderedList);
		expect(result.tokens[0].children).toHaveLength(3);
		expect(result.tokens[0].children![0].type).toBe(TokenType.ListItem);
	});

	test('parses ordered list', () => {
		const source = '1. First\n2. Second\n3. Third';
		const result = parse(source);

		expect(result.tokens).toHaveLength(1);
		expect(result.tokens[0].type).toBe(TokenType.OrderedList);
		expect(result.tokens[0].children).toHaveLength(3);
		expect(result.tokens[0].children![0].type).toBe(TokenType.ListItem);
	});

	test('parses blockquote', () => {
		const source = '> This is a quote';
		const result = parse(source);

		expect(result.tokens).toHaveLength(1);
		expect(result.tokens[0].type).toBe(TokenType.Blockquote);
		expect(result.tokens[0].children!.length).toBeGreaterThan(0);
	});

	test('parses horizontal rule', () => {
		const source = '---';
		const result = parse(source);

		expect(result.tokens).toHaveLength(1);
		expect(result.tokens[0].type).toBe(TokenType.HorizontalRule);
	});

	test('parses inline LaTeX', () => {
		const source = 'Formula: $x$';
		const result = parse(source);

		expect(result.tokens).toHaveLength(1);
		expect(result.tokens[0].type).toBe(TokenType.Paragraph);

		const children = result.tokens[0].children || [];
		const latex = children.find((t) => t.type === TokenType.LatexInline);

		expect(latex).toBeDefined();
		expect((latex as any).content).toBe('x');
	});

	test('parses block LaTeX', () => {
		const source = '$$\nx^2 + y^2 = z^2\n$$';
		const result = parse(source);

		expect(result.tokens).toHaveLength(1);
		expect(result.tokens[0].type).toBe(TokenType.LatexBlock);
		expect((result.tokens[0] as any).content).toBe('x^2 + y^2 = z^2');
	});

	test('parses table', () => {
		const source = '| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |';
		const result = parse(source);

		expect(result.tokens).toHaveLength(1);
		expect(result.tokens[0].type).toBe(TokenType.Table);
		expect(result.tokens[0].children!.length).toBeGreaterThan(0);
		expect(result.tokens[0].children![0].type).toBe(TokenType.TableRow);
	});

	test('parses citation', () => {
		const source = '[@citation123]';
		const result = parse(source);

		expect(result.tokens).toHaveLength(1);
		expect(result.tokens[0].type).toBe(TokenType.Paragraph);

		const children = result.tokens[0].children || [];
		const citation = children.find((t) => t.type === TokenType.Citation);

		expect(citation).toBeDefined();
		expect((citation as any).id).toBe('citation123');
	});

	test('parses mixed content document', () => {
		const source = `# Title

This is a paragraph with **bold** text.

- List item 1
- List item 2

\`\`\`javascript
const x = 1;
\`\`\`

> A quote

[A link](https://example.com)`;

		const result = parse(source);

		expect(result.tokens.length).toBeGreaterThan(5);
		expect(result.tokens.some((t) => t.type === TokenType.Heading)).toBe(true);
		expect(result.tokens.some((t) => t.type === TokenType.Paragraph)).toBe(true);
		expect(result.tokens.some((t) => t.type === TokenType.UnorderedList)).toBe(true);
		expect(result.tokens.some((t) => t.type === TokenType.CodeBlock)).toBe(true);
		expect(result.tokens.some((t) => t.type === TokenType.Blockquote)).toBe(true);
	});

	test('provides region boundaries for complex structures', () => {
		const source = '```js\ncode\n```\n\n> quote\n\n- list';
		const result = parse(source);

		expect(result.regions.length).toBeGreaterThan(0);
		expect(result.regions.some((r) => r.type === 'codeblock')).toBe(true);
	});

	test('handles empty input', () => {
		const source = '';
		const result = parse(source);

		expect(result.tokens).toHaveLength(0);
		expect(result.regions).toHaveLength(0);
	});

	test('handles whitespace-only input', () => {
		const source = '   \n\n   ';
		const result = parse(source);

		expect(result.tokens).toHaveLength(0);
	});

	test('should not create extra links around code blocks after horizontal rules', () => {
		const source = `---
3. **Generate candidates**  
   For each prompt, run a *grid* of parameter sets.  
   \`\`\`python
   params_list = [
       {"temperature":0.2,"top_p":0.8,"max_tokens":8,"top_k":50},
       {"temperature":0.5,"top_p":0.9,"max_tokens":10,"top_k":100},
       # … add more combos
   ]
   \`\`\`
---`;

		const result = parse(source);

		// Should have: HorizontalRule, OrderedList, Paragraph, CodeBlock, HorizontalRule
		expect(result.tokens).toHaveLength(5);
		expect(result.tokens[0].type).toBe(TokenType.HorizontalRule);
		expect(result.tokens[1].type).toBe(TokenType.OrderedList);
		expect(result.tokens[2].type).toBe(TokenType.Paragraph);
		expect(result.tokens[3].type).toBe(TokenType.CodeBlock);
		expect(result.tokens[4].type).toBe(TokenType.HorizontalRule);

		// Check that no extra Link tokens are created
		let linkCount = 0;
		function countLinks(tokens: any[]) {
			for (const token of tokens) {
				if (token.type === TokenType.Link) {
					linkCount++;
				}
				if (token.children) {
					countLinks(token.children);
				}
			}
		}
		countLinks(result.tokens);

		expect(linkCount).toBe(0); // There should be no links in this markdown
	});
});
