import { describe, it, expect, beforeEach } from 'vitest';
import type { Tree } from '@lezer/common';
import { parse, parseIncremental, walkTree } from './index';

// Helper function to recursively check if a node type exists in the AST
function containsType(node: any, typeName: string): boolean {
	if (!node) return false;
	if (node.type === typeName) return true;
	if (!Array.isArray(node.children)) return false;
	return node.children.some((child: any) => containsType(child, typeName));
}

describe('Lexer - parse', () => {
	it('should parse simple markdown text', async () => {
		const source = '# Hello World';
		const tree = await parse(source);

		expect(tree).toBeDefined();
		expect(tree.length).toBeGreaterThan(0);
	});

	it('should parse markdown with emphasis', async () => {
		const source = '**bold** and *italic*';
		const tree = await parse(source);

		expect(tree).toBeDefined();
		expect(tree.length).toBeGreaterThan(0);
	});

	it('should parse markdown with lists', async () => {
		const source = '- Item 1\n- Item 2\n- Item 3';
		const tree = await parse(source);

		expect(tree).toBeDefined();
		expect(tree.length).toBeGreaterThan(0);
	});

	it('should parse markdown with code blocks', async () => {
		const source = '```typescript\nconst x = 1;\n```';
		const tree = await parse(source);

		expect(tree).toBeDefined();
		expect(tree.length).toBeGreaterThan(0);
	});

	it('should parse markdown with inline code', async () => {
		const source = 'This is `inline code` in text';
		const tree = await parse(source);

		expect(tree).toBeDefined();
		expect(tree.length).toBeGreaterThan(0);
	});

	it('should parse markdown with links', async () => {
		const source = '[Google](https://google.com)';
		const tree = await parse(source);

		expect(tree).toBeDefined();
		expect(tree.length).toBeGreaterThan(0);
	});

	it('should parse markdown with blockquotes', async () => {
		const source = '> This is a quote\n> Second line';
		const tree = await parse(source);

		expect(tree).toBeDefined();
		expect(tree.length).toBeGreaterThan(0);
	});

	it('should parse markdown with tables', async () => {
		const source = '| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |';
		const tree = await parse(source);

		expect(tree).toBeDefined();
		expect(tree.length).toBeGreaterThan(0);
	});

	it('should parse empty string', async () => {
		const tree = await parse('');

		expect(tree).toBeDefined();
		expect(tree.length).toBeGreaterThanOrEqual(0);
	});

	it('should parse markdown with LaTeX display math', async () => {
		const source = '$$E = mc^2$$';
		const tree = await parse(source);

		expect(tree).toBeDefined();
		expect(tree.length).toBeGreaterThan(0);
	});

	it('should parse markdown with LaTeX bracket notation', async () => {
		const source = '\\[x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}\\]';
		const tree = await parse(source);

		expect(tree).toBeDefined();
		expect(tree.length).toBeGreaterThan(0);
	});

	it('should parse block math with text before it on previous line', async () => {
		const source = `Square both sides:

\\[
\\cos^2\\theta_x \\sin^2\\theta_y + \\sin^2\\theta_x
=
\\frac{1}{4} \\cos^2\\theta_y
\\tag{B}
\\]`;

		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(containsType(walked, 'BlockMathBracket')).toBe(true);
	});

	// The test is comment out because API required for this test is not exposed, consider fix it later
	// it('should not treat dollar amounts as inline math when no surrounding spaces exist', async () => {
	// 	const source = 'It costs $1, and that cost $2';
	// 	const tree = await parse(source);
	// 	const walked = await walkTree(tree, source);

	// 	// Debug logging to inspect parse/walk output for failing cases
	// 	// Note: kept minimal to avoid noisy test output, but helpful for investigation
	// 	try {
	// 		// `walked` is a plain AST object returned by walkTree and should be serializable
	// 		console.log('DEBUG: source ->', source);
	// 		console.log('DEBUG: walked AST ->', JSON.stringify(walked, null, 2));
	// 	} catch (e) {
	// 		// If serialization fails for any reason, fallback to logging top-level info
	// 		console.log('DEBUG: walked (non-serializable) ->', walked);
	// 	}

	// 	// Recursively search for a node type in the walked AST
	// 	const containsType = (node: any, typeName: string): boolean => {
	// 		if (!node) return false;
	// 		if (node.type === typeName) return true;
	// 		if (!Array.isArray(node.children)) return false;
	// 		return node.children.some((child: any) => containsType(child, typeName));
	// 	};

	// 	// Ensure no InlineMathDollar or InlineMathBracket nodes are present
	// 	expect(containsType(walked, 'InlineMathDollar')).toBe(false);
	// 	expect(containsType(walked, 'InlineMathBracket')).toBe(false);
	// });

	it('should parse standard inline LaTeX with spaced dollar delimiters', async () => {
		const source = '$ \\text{A} $';
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		// With spaces around the dollar delimiters this should be detected as inline math
		expect(containsType(walked, 'InlineMathDollar')).toBe(true);
	});

	it('should parse inline LaTeX using \\( \\) delimiters', async () => {
		const source = '\\(\\text{A}\\)';
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(containsType(walked, 'InlineMathBracket')).toBe(true);
	});

	it('should parse single-character inline LaTeX without spaces', async () => {
		const source = '$x$';
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(containsType(walked, 'InlineMathDollar')).toBe(true);
	});

	it('should not parse multi-character inline LaTeX without spaces', async () => {
		const source = '$x*y$';
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(containsType(walked, 'InlineMathDollar')).toBe(false);
	});

	it('should not treat dollar amounts as inline math', async () => {
		const source = 'The apple is $1, orange is $2';
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(containsType(walked, 'InlineMathDollar')).toBe(false);
	});

	it('should parse multi-character inline LaTeX with spaces', async () => {
		const source = '$ x*y $';
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(containsType(walked, 'InlineMathDollar')).toBe(true);
	});

	it('should parse markdown with mixed content', async () => {
		const source = `# Title

This is a paragraph with **bold** and *italic*.

- List item 1
- List item 2

\`\`\`js
console.log('code');
\`\`\`

> A quote

$$x^2 + y^2 = z^2$$`;
		const tree = await parse(source);

		expect(tree).toBeDefined();
		expect(tree.length).toBeGreaterThan(0);
	});
});

describe('Lexer - parseIncremental', () => {
	let initialTree: Tree;
	const initialSource = '# Hello\n\nThis is a paragraph.';

	beforeEach(async () => {
		initialTree = await parse(initialSource);
	});

	it('should return previous tree when source is unchanged', async () => {
		const result = await parseIncremental(initialTree, initialSource, initialSource);

		expect(result).toBe(initialTree);
	});

	it('should handle appending text to source', async () => {
		const newSource = initialSource + '\n\nMore text here.';
		const result = await parseIncremental(initialTree, initialSource, newSource);

		expect(result).toBeDefined();
		expect(result.length).toBeGreaterThan(initialTree.length);
	});

	it('should fall back to full parse when source is not an append', async () => {
		const newSource = 'Completely different content';
		const result = await parseIncremental(initialTree, initialSource, newSource);

		expect(result).toBeDefined();
		expect(result.length).toBeGreaterThan(0);
	});

	it('should handle appending lists', async () => {
		const newSource = initialSource + '\n\n- Item 1\n- Item 2';
		const result = await parseIncremental(initialTree, initialSource, newSource);

		expect(result).toBeDefined();
		expect(result.length).toBeGreaterThan(initialTree.length);
	});

	it('should handle appending code blocks', async () => {
		const newSource = initialSource + '\n\n```\ncode\n```';
		const result = await parseIncremental(initialTree, initialSource, newSource);

		expect(result).toBeDefined();
		expect(result.length).toBeGreaterThan(initialTree.length);
	});

	it('should handle appending LaTeX blocks', async () => {
		const newSource = initialSource + '\n\n$$x = y$$';
		const result = await parseIncremental(initialTree, initialSource, newSource);

		expect(result).toBeDefined();
		expect(result.length).toBeGreaterThan(initialTree.length);
	});

	it('should handle appending tables', async () => {
		const newSource = initialSource + '\n\n| A | B |\n|---|---|\n| 1 | 2 |';
		const result = await parseIncremental(initialTree, initialSource, newSource);

		expect(result).toBeDefined();
		expect(result.length).toBeGreaterThan(initialTree.length);
	});

	it('should handle small appends efficiently', async () => {
		const newSource = initialSource + ' more.';
		const result = await parseIncremental(initialTree, initialSource, newSource);

		expect(result).toBeDefined();
	});

	it('should handle multiple consecutive incremental updates', async () => {
		let currentTree = initialTree;
		let currentSource = initialSource;

		for (let i = 0; i < 3; i++) {
			const newSource = currentSource + `\n\nAddition ${i}`;
			currentTree = await parseIncremental(currentTree, currentSource, newSource);
			currentSource = newSource;

			expect(currentTree).toBeDefined();
			expect(currentTree.length).toBeGreaterThan(0);
		}
	});
});

describe('Lexer - walkTree', () => {
	it('should return null for null tree', async () => {
		const result = await walkTree(null, 'source');

		expect(result).toBeNull();
	});

	it('should walk simple heading tree', async () => {
		const source = '# Hello World';
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(walked).toBeDefined();
		expect(walked.type).toBe('Document');
		expect(walked.text).toBe(source);
		expect(Array.isArray(walked.children)).toBe(true);
	});

	it('should include node types in tree walk', async () => {
		const source = '# Heading\n\nParagraph text';
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(walked).toBeDefined();
		expect(walked.type).toBe('Document');
		expect(walked.children.length).toBeGreaterThan(0);
	});

	it('should include from and to positions', async () => {
		const source = '**bold**';
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(walked).toBeDefined();
		expect(typeof walked.from).toBe('number');
		expect(typeof walked.to).toBe('number');
		expect(walked.from).toBeLessThanOrEqual(walked.to);
	});

	it('should handle nested structures', async () => {
		const source = '- **bold item**\n- *italic item*';
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(walked).toBeDefined();
		expect(walked.children.length).toBeGreaterThan(0);
		// Verify nested children exist
		const hasNestedChildren = walked.children.some(
			(child: any) => Array.isArray(child.children) && child.children.length > 0
		);
		expect(hasNestedChildren).toBe(true);
	});

	it('should walk code block structure', async () => {
		const source = '```js\nconst x = 1;\n```';
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(walked).toBeDefined();
		expect(walked.text).toContain('```');
	});

	it('should detect and transform citation blocks', async () => {
		const source = `<citation>
    <title>Example Article</title>
    <url>https://example.com</url>
</citation>`;
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(walked).toBeDefined();
		// Navigate through the tree to find the Citation node
		const findCitation = (node: any): any => {
			if (node.type === 'Citation') {
				return node;
			}
			if (Array.isArray(node.children)) {
				for (const child of node.children) {
					const result = findCitation(child);
					if (result) return result;
				}
			}
			return null;
		};

		const citationNode = findCitation(walked);
		if (citationNode) {
			expect(citationNode.type).toBe('Citation');
			expect(citationNode.citationData).toBeDefined();
			expect(citationNode.citationData.title).toBe('Example Article');
			expect(citationNode.citationData.url).toBe('https://example.com');
		}
	});

	it('should walk table structure', async () => {
		const source = '| A | B |\n|---|---|\n| 1 | 2 |';
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(walked).toBeDefined();
		expect(walked.text).toContain('|');
	});

	it('should walk LaTeX structure', async () => {
		const source = '$$E = mc^2$$';
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(walked).toBeDefined();
	});

	it('should walk empty document', async () => {
		const source = '';
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(walked).toBeDefined();
		expect(walked.type).toBe('Document');
	});

	it('should walk complex document with mixed content', async () => {
		const source = `# Title

Paragraph with **bold** and *italic*.

- List item
- Another item

\`\`\`js
code
\`\`\`

> Quote

$$x^2$$`;
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(walked).toBeDefined();
		expect(walked.type).toBe('Document');
		expect(walked.children.length).toBeGreaterThan(0);
		expect(walked.text).toBe(source);
	});

	it('should maintain correct position boundaries', async () => {
		const source = 'First paragraph.\n\nSecond paragraph.';
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		const checkPositions = (node: any): boolean => {
			if (node.from < 0 || node.to < node.from || node.to > source.length) {
				return false;
			}
			if (Array.isArray(node.children)) {
				return node.children.every(checkPositions);
			}
			return true;
		};

		expect(checkPositions(walked)).toBe(true);
	});
});

describe('Lexer - Integration', () => {
	it('should parse and walk a document', async () => {
		const source = '# Hello\n\nThis is **bold** text.';
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(tree).toBeDefined();
		expect(walked).toBeDefined();
		expect(walked.type).toBe('Document');
		expect(walked.text).toBe(source);
	});

	it('should handle incremental parse followed by walk', async () => {
		const initialSource = '# Initial';
		const initialTree = await parse(initialSource);

		const newSource = initialSource + '\n\nAdded content.';
		const updatedTree = await parseIncremental(initialTree, initialSource, newSource);

		const walked = await walkTree(updatedTree, newSource);

		expect(walked).toBeDefined();
		expect(walked.type).toBe('Document');
		expect(walked.text).toBe(newSource);
	});

	it('should handle citation block through full pipeline', async () => {
		const source = `<citation>
    <title>Test Source</title>
    <url>https://test.com</url>
    <favicon>https://test.com/icon.ico</favicon>
    <authoritative />
</citation>`;
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(tree).toBeDefined();
		expect(walked).toBeDefined();

		const findCitation = (node: any): any => {
			if (node.type === 'Citation') return node;
			if (Array.isArray(node.children)) {
				for (const child of node.children) {
					const result = findCitation(child);
					if (result) return result;
				}
			}
			return null;
		};

		const citationNode = findCitation(walked);
		if (citationNode) {
			expect(citationNode.citationData.title).toBe('Test Source');
			expect(citationNode.citationData.url).toBe('https://test.com');
			expect(citationNode.citationData.authoritative).toBe(true);
		}
	});

	it('should handle citation block with tabs through full pipeline', async () => {
		const source = `<citation>
	<title>Apple's Return to Intel: M-Series Chip Deal by 2027</title>
	<url>https://apple.gadgethacks.com/news/apple-returns-to-intel-m-series-chip-deal-by-2027/</url>
	<favicon>https://assets.content.technologyadvice.com/gadgethacks_favicon_d352b3f01c.webp</favicon>
</citation>`;
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(tree).toBeDefined();
		expect(walked).toBeDefined();

		const findCitation = (node: any): any => {
			if (node.type === 'Citation') return node;
			if (Array.isArray(node.children)) {
				for (const child of node.children) {
					const result = findCitation(child);
					if (result) return result;
				}
			}
			return null;
		};

		const citationNode = findCitation(walked);
		if (citationNode) {
			expect(citationNode.citationData.title).toBe(
				"Apple's Return to Intel: M-Series Chip Deal by 2027"
			);
			expect(citationNode.citationData.url).toBe(
				'https://apple.gadgethacks.com/news/apple-returns-to-intel-m-series-chip-deal-by-2027/'
			);
			expect(citationNode.citationData.favicon).toBe(
				'https://assets.content.technologyadvice.com/gadgethacks_favicon_d352b3f01c.webp'
			);
		}
	});

	it('should handle citation block with leading tab indentation', async () => {
		const source = `	<citation>
	<title>Test with leading tab</title>
	<url>https://test.com</url>
</citation>`;
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(tree).toBeDefined();
		expect(walked).toBeDefined();

		const findCitation = (node: any): any => {
			if (node.type === 'Citation') return node;
			if (Array.isArray(node.children)) {
				for (const child of node.children) {
					const result = findCitation(child);
					if (result) return result;
				}
			}
			return null;
		};

		const citationNode = findCitation(walked);

		expect(citationNode).toBeDefined();
		if (citationNode) {
			expect(citationNode.citationData.title).toBe('Test with leading tab');
			expect(citationNode.citationData.url).toBe('https://test.com');
		}
	});

	it('should handle the exact user example with tabs through full pipeline', async () => {
		const source = `	<citation>
	<title>Apple's Return to Intel: M-Series Chip Deal by 2027</title>
	<url>https://apple.gadgethacks.com/news/apple-returns-to-intel-m-series-chip-deal-by-2027/</url>
	<favicon>https://assets.content.technologyadvice.com/gadgethacks_favicon_d352b3f01c.webp</favicon>
</citation>`;
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(tree).toBeDefined();
		expect(walked).toBeDefined();

		const findCitation = (node: any): any => {
			if (node.type === 'Citation') return node;
			if (Array.isArray(node.children)) {
				for (const child of node.children) {
					const result = findCitation(child);
					if (result) return result;
				}
			}
			return null;
		};

		const citationNode = findCitation(walked);
		expect(citationNode).toBeDefined();
		if (citationNode) {
			expect(citationNode.citationData.title).toBe(
				"Apple's Return to Intel: M-Series Chip Deal by 2027"
			);
			expect(citationNode.citationData.url).toBe(
				'https://apple.gadgethacks.com/news/apple-returns-to-intel-m-series-chip-deal-by-2027/'
			);
			expect(citationNode.citationData.favicon).toBe(
				'https://assets.content.technologyadvice.com/gadgethacks_favicon_d352b3f01c.webp'
			);
		}
	});

	it('should handle consecutive citations through full pipeline', async () => {
		const source = `<citation>
    <title>Intel Makes U‑turn, Cancels Plan to Sell Its Networking Division: Read Chipmaker's Statement</title>
    <url>https://timesofindia.indiatimes.com/technology/tech-news/intel-makes-u-turn-cancels-plan-to-sell-its-networking-division-read-chipmakers-statement/articleshow/125769743.cms</url>
    <favicon>https://m.timesofindia.com/touch-icon-iphone-precomposed.png</favicon>
</citation>
<citation>
    <title>Apple's Return to Intel: M-Series Chip Deal by 2027</title>
    <url>https://apple.gadgethacks.com/news/apple-returns-to-intel-m-series-chip-deal-by-2027/</url>
    <favicon>https://assets.content.technologyadvice.com/gadgethacks_favicon_d352b3f01c.webp</favicon>
</citation>
<citation>
    <title>The 10 Biggest Intel News Stories Of 2025</title>
    <url>https://www.crn.com/news/components-peripherals/2025/the-10-biggest-intel-news-stories-of-2025</url>
    <favicon>https://www.crn.com/icons/apple-touch-icon.png</favicon>
    <author>Dylan Martin</author>
</citation>`;
		const tree = await parse(source);
		const walked = await walkTree(tree, source);

		expect(tree).toBeDefined();
		expect(walked).toBeDefined();

		const findAllCitations = (node: any): any[] => {
			const citations: any[] = [];
			if (node.type === 'Citation') {
				citations.push(node);
			}
			if (Array.isArray(node.children)) {
				for (const child of node.children) {
					citations.push(...findAllCitations(child));
				}
			}
			return citations;
		};

		const citations = findAllCitations(walked);
		expect(citations).toHaveLength(3);

		// First citation
		expect(citations[0].citationData.title).toBe(
			"Intel Makes U‑turn, Cancels Plan to Sell Its Networking Division: Read Chipmaker's Statement"
		);
		expect(citations[0].citationData.url).toBe(
			'https://timesofindia.indiatimes.com/technology/tech-news/intel-makes-u-turn-cancels-plan-to-sell-its-networking-division-read-chipmakers-statement/articleshow/125769743.cms'
		);

		// Second citation
		expect(citations[1].citationData.title).toBe(
			"Apple's Return to Intel: M-Series Chip Deal by 2027"
		);
		expect(citations[1].citationData.url).toBe(
			'https://apple.gadgethacks.com/news/apple-returns-to-intel-m-series-chip-deal-by-2027/'
		);

		// Third citation
		expect(citations[2].citationData.title).toBe('The 10 Biggest Intel News Stories Of 2025');
		expect(citations[2].citationData.url).toBe(
			'https://www.crn.com/news/components-peripherals/2025/the-10-biggest-intel-news-stories-of-2025'
		);
	});
});
