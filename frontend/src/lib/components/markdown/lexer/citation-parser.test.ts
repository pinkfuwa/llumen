import { describe, it, expect } from 'vitest';
import { parseCitation, isCitationBlock, splitCitations } from './citation-parser';

describe('parseCitation', () => {
	it('should parse a complete citation block', () => {
		const input = `<citation>
    <title>Example Article</title>
    <url>https://example.com/article</url>
    <favicon>https://example.com/favicon.ico</favicon>
    <authoritative />
</citation>`;

		const result = parseCitation(input);

		expect(result.title).toBe('Example Article');
		expect(result.url).toBe('https://example.com/article');
		expect(result.favicon).toBe('https://example.com/favicon.ico');
		expect(result.authoritative).toBe(true);
		expect(result.raw).toBe(input);
	});

	it('should parse citation without authoritative tag', () => {
		const input = `<citation>
    <title>Regular Source</title>
    <url>https://example.com</url>
    <favicon>https://example.com/icon.png</favicon>
</citation>`;

		const result = parseCitation(input);

		expect(result.title).toBe('Regular Source');
		expect(result.url).toBe('https://example.com');
		expect(result.favicon).toBe('https://example.com/icon.png');
		expect(result.authoritative).toBeUndefined();
	});

	it('should parse citation with missing optional fields', () => {
		const input = `<citation>
    <title>Minimal Citation</title>
    <url>https://example.com</url>
</citation>`;

		const result = parseCitation(input);

		expect(result.title).toBe('Minimal Citation');
		expect(result.url).toBe('https://example.com');
		expect(result.favicon).toBeUndefined();
		expect(result.authoritative).toBeUndefined();
	});

	it('should handle multiline content in tags', () => {
		const input = `<citation>
    <title>Very Long Title
    That Spans Multiple Lines</title>
    <url>https://example.com</url>
</citation>`;

		const result = parseCitation(input);

		expect(result.title).toContain('Very Long Title');
		expect(result.title).toContain('That Spans Multiple Lines');
	});

	it('should handle authoritative tag without self-closing', () => {
		const input = `<citation>
    <title>Test</title>
    <url>https://example.com</url>
    <authoritative>
</citation>`;

		const result = parseCitation(input);

		expect(result.authoritative).toBe(true);
	});
});

describe('isCitationBlock', () => {
	it('should return true for valid citation blocks', () => {
		const input = `<citation>
    <title>Test</title>
</citation>`;

		expect(isCitationBlock(input)).toBe(true);
	});

	it('should return true for citation with leading whitespace', () => {
		const input = `   <citation>
    <title>Test</title>
</citation>`;

		expect(isCitationBlock(input)).toBe(true);
	});

	it('should return false for non-citation blocks', () => {
		const input = `<div>Not a citation</div>`;

		expect(isCitationBlock(input)).toBe(false);
	});

	it('should return false for incomplete citation blocks', () => {
		const input = `<citation>
    <title>Test</title>`;

		expect(isCitationBlock(input)).toBe(false);
	});
});

describe('splitCitations', () => {
	it('should split multiple citation blocks', () => {
		const input = `<citation>
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

		const result = splitCitations(input, 0);

		expect(result).toHaveLength(3);
		expect(result[0].text).toContain('First Citation');
		expect(result[1].text).toContain('Second Citation');
		expect(result[2].text).toContain('Third Citation');
	});

	it('should handle single citation block', () => {
		const input = `<citation>
    <title>Single Citation</title>
    <url>https://example.com</url>
</citation>`;

		const result = splitCitations(input, 0);

		expect(result).toHaveLength(1);
		expect(result[0].text).toContain('Single Citation');
	});

	it('should correctly calculate positions with offset', () => {
		const input = `<citation>
    <title>First</title>
</citation>
<citation>
    <title>Second</title>
</citation>`;

		const result = splitCitations(input, 100);

		expect(result).toHaveLength(2);
		expect(result[0].from).toBe(100);
		expect(result[0].to).toBeGreaterThan(100);
		expect(result[1].from).toBeGreaterThan(result[0].to);
	});

	it('should handle unclosed citation at end', () => {
		const input = `<citation>
    <title>Unclosed Citation</title>
    <url>https://example.com</url>`;

		const result = splitCitations(input, 0);

		expect(result).toHaveLength(1);
		expect(result[0].text).toContain('Unclosed Citation');
		expect(result[0].to).toBe(input.length);
	});

	it('should handle real-world example from bug report', () => {
		const input = `<citation>
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

		const result = splitCitations(input, 0);

		expect(result).toHaveLength(8);

		// Verify each citation is parsed correctly
		const citations = result.map((r) => parseCitation(r.text));
		expect(citations[0].title).toBe('Six issues that will dominate COP30');
		expect(citations[1].title).toBe('COP30 Evening Summary – November 10');
		expect(citations[2].title).toBe(
			"Key takeaways from the COP30 Circle of Finance Minister's report"
		);
		expect(citations[3].title).toBe(
			'5 things you should know about the COP30 UN Climate Conference'
		);
		expect(citations[4].title).toBe(
			'Q&A: what are the main issues at Cop30 and why do they matter?'
		);
		expect(citations[5].title).toBe('UN Climate Change Conference - Belém, November 2025');
		expect(citations[6].title).toBe('What is COP30 and why does it matter?');
		expect(citations[7].title).toBe('COP30 Evening Summary – November 12');
	});
});
