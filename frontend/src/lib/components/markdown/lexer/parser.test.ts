import { describe, it, expect } from 'vitest';
import { parse, walkTree, parseIncremental } from './parser';

describe('Multiple citations parsing', () => {
	it('should parse multiple consecutive citation blocks correctly', () => {
		const input = `As COP30 progresses, these initial launches and commitments signal a people-focused approach, but comprehensive agreements on emissions targets, loss and damage finance, and forest protection are expected in the summit's final days around November 21.

<citation>
    <title>Six issues that will dominate COP30</title>
    <url>https://www.unep.org/news-and-stories/story/six-issues-will-dominate-cop30</url>
</citation>
<citation>
    <title>COP30 Evening Summary – November 10</title>
    <url>https://cop30.br/en/news-about-cop30/cop30-evening-summary</url>
</citation>
<citation>
    <title>Key takeaways from the COP30 Circle of Finance Minister's report</title>
    <url>https://www.atlanticcouncil.org/blogs/energysource/key-takeaways-from-the-cop30-circle-of-finance-ministers-report/</url>
</citation>
<citation>
    <title>5 things you should know about the COP30 UN Climate Conference</title>
    <url>https://climate.ec.europa.eu/news-other-reads/news/5-things-you-should-know-about-cop30-un-climate-conference-2025-11-07_en</url>
</citation>
<citation>
    <title>Q&A: what are the main issues at Cop30 and why do they matter?</title>
    <url>https://www.theguardian.com/environment/2025/nov/10/cop30-what-are-the-main-issues-and-why-do-they-matter</url>
</citation>
<citation>
    <title>UN Climate Change Conference - Belém, November 2025</title>
    <url>https://unfccc.int/cop30</url>
</citation>
<citation>
    <title>What is COP30 and why does it matter?</title>
    <url>https://www.cnn.com/2025/11/11/climate/cop30-explainer-belem-brazil</url>
</citation>
<citation>
    <title>COP30 Evening Summary – November 12</title>
    <url>https://cop30.br/en/news-about-cop30/cop30-evening-summary-november-12</url>
</citation>`;

		const tree = parse(input);
		const ast = walkTree(tree, input);

		// Helper function to find all Citation nodes
		function findCitations(node: any): any[] {
			const citations: any[] = [];
			if (node.type === 'Citation') {
				citations.push(node);
			}
			if (node.children) {
				for (const child of node.children) {
					citations.push(...findCitations(child));
				}
			}
			return citations;
		}

		const citations = findCitations(ast);

		// Should find all 8 citations
		expect(citations).toHaveLength(8);

		// Verify each citation has correct data
		expect(citations[0].citationData.title).toBe('Six issues that will dominate COP30');
		expect(citations[0].citationData.url).toBe(
			'https://www.unep.org/news-and-stories/story/six-issues-will-dominate-cop30'
		);

		expect(citations[1].citationData.title).toBe('COP30 Evening Summary – November 10');
		expect(citations[1].citationData.url).toBe(
			'https://cop30.br/en/news-about-cop30/cop30-evening-summary'
		);

		expect(citations[2].citationData.title).toBe(
			"Key takeaways from the COP30 Circle of Finance Minister's report"
		);
		expect(citations[2].citationData.url).toBe(
			'https://www.atlanticcouncil.org/blogs/energysource/key-takeaways-from-the-cop30-circle-of-finance-ministers-report/'
		);

		expect(citations[3].citationData.title).toBe(
			'5 things you should know about the COP30 UN Climate Conference'
		);

		expect(citations[4].citationData.title).toBe(
			'Q&A: what are the main issues at Cop30 and why do they matter?'
		);

		expect(citations[5].citationData.title).toBe(
			'UN Climate Change Conference - Belém, November 2025'
		);

		expect(citations[6].citationData.title).toBe('What is COP30 and why does it matter?');

		expect(citations[7].citationData.title).toBe('COP30 Evening Summary – November 12');
		expect(citations[7].citationData.url).toBe(
			'https://cop30.br/en/news-about-cop30/cop30-evening-summary-november-12'
		);
	});

	it('should parse single citation block correctly', () => {
		const input = `Some text before

<citation>
    <title>Single Citation</title>
    <url>https://example.com</url>
</citation>

Some text after`;

		const tree = parse(input);
		const ast = walkTree(tree, input);

		function findCitations(node: any): any[] {
			const citations: any[] = [];
			if (node.type === 'Citation') {
				citations.push(node);
			}
			if (node.children) {
				for (const child of node.children) {
					citations.push(...findCitations(child));
				}
			}
			return citations;
		}

		const citations = findCitations(ast);

		expect(citations).toHaveLength(1);
		expect(citations[0].citationData.title).toBe('Single Citation');
		expect(citations[0].citationData.url).toBe('https://example.com');
	});

	it('should parse citations separated by text correctly', () => {
		const input = `First paragraph

<citation>
    <title>First Citation</title>
    <url>https://example.com/first</url>
</citation>

Middle paragraph

<citation>
    <title>Second Citation</title>
    <url>https://example.com/second</url>
</citation>

Last paragraph`;

		const tree = parse(input);
		const ast = walkTree(tree, input);

		function findCitations(node: any): any[] {
			const citations: any[] = [];
			if (node.type === 'Citation') {
				citations.push(node);
			}
			if (node.children) {
				for (const child of node.children) {
					citations.push(...findCitations(child));
				}
			}
			return citations;
		}

		const citations = findCitations(ast);

		expect(citations).toHaveLength(2);
		expect(citations[0].citationData.title).toBe('First Citation');
		expect(citations[1].citationData.title).toBe('Second Citation');
	});
});

describe('Incremental parsing with multiple citations', () => {
	it('should incrementally parse appended citations correctly', () => {
		// Start with initial text
		const initialText = `Some initial text.

<citation>
    <title>First Citation</title>
    <url>https://example.com/first</url>
</citation>`;

		const initialTree = parse(initialText);
		const initialAst = walkTree(initialTree, initialText);

		function findCitations(node: any): any[] {
			const citations: any[] = [];
			if (node.type === 'Citation') {
				citations.push(node);
			}
			if (node.children) {
				for (const child of node.children) {
					citations.push(...findCitations(child));
				}
			}
			return citations;
		}

		let citations = findCitations(initialAst);
		expect(citations).toHaveLength(1);
		expect(citations[0].citationData.title).toBe('First Citation');

		// Append more citations
		const appendedText = `Some initial text.

<citation>
    <title>First Citation</title>
    <url>https://example.com/first</url>
</citation>
<citation>
    <title>Second Citation</title>
    <url>https://example.com/second</url>
</citation>
<citation>
    <title>Third Citation</title>
    <url>https://example.com/third</url>
</citation>`;

		const incrementalTree = parseIncremental(initialTree, initialText, appendedText);
		const incrementalAst = walkTree(incrementalTree, appendedText);

		citations = findCitations(incrementalAst);
		expect(citations).toHaveLength(3);
		expect(citations[0].citationData.title).toBe('First Citation');
		expect(citations[1].citationData.title).toBe('Second Citation');
		expect(citations[2].citationData.title).toBe('Third Citation');
	});

	it('should handle incrementally appending citations one by one', () => {
		let text = 'Initial paragraph.\n\n';
		let tree = parse(text);

		// Append first citation
		text += `<citation>
    <title>Citation One</title>
    <url>https://example.com/one</url>
</citation>\n`;
		tree = parseIncremental(tree, text.slice(0, -text.length + tree.length), text);

		let ast = walkTree(tree, text);
		let citations = findCitations(ast);

		function findCitations(node: any): any[] {
			const citations: any[] = [];
			if (node.type === 'Citation') {
				citations.push(node);
			}
			if (node.children) {
				for (const child of node.children) {
					citations.push(...findCitations(child));
				}
			}
			return citations;
		}

		expect(citations.length).toBeGreaterThanOrEqual(1);
		expect(citations[0].citationData.title).toBe('Citation One');

		// Append second citation
		const prevText = text;
		text += `<citation>
    <title>Citation Two</title>
    <url>https://example.com/two</url>
</citation>\n`;
		tree = parseIncremental(tree, prevText, text);

		ast = walkTree(tree, text);
		citations = findCitations(ast);

		expect(citations.length).toBeGreaterThanOrEqual(2);
		expect(citations[0].citationData.title).toBe('Citation One');
		expect(citations[1].citationData.title).toBe('Citation Two');
	});
});
