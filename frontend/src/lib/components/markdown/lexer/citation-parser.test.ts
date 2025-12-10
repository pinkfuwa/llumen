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

	it('should parse citation with tabs instead of spaces', () => {
		const input = `<citation>
	<title>Apple's Return to Intel: M-Series Chip Deal by 2027</title>
	<url>https://apple.gadgethacks.com/news/apple-returns-to-intel-m-series-chip-deal-by-2027/</url>
	<favicon>https://assets.content.technologyadvice.com/gadgethacks_favicon_d352b3f01c.webp</favicon>
</citation>`;

		const result = parseCitation(input);

		expect(result.title).toBe("Apple's Return to Intel: M-Series Chip Deal by 2027");
		expect(result.url).toBe(
			'https://apple.gadgethacks.com/news/apple-returns-to-intel-m-series-chip-deal-by-2027/'
		);
		expect(result.favicon).toBe(
			'https://assets.content.technologyadvice.com/gadgethacks_favicon_d352b3f01c.webp'
		);
		expect(result.raw).toBe(input);
	});

	it('should detect citation block with tabs', () => {
		const input = `<citation>
	<title>Test</title>
</citation>`;

		expect(isCitationBlock(input)).toBe(true);
	});

	it('should parse the exact user example with tabs', () => {
		const input = `<citation>
	<title>Apple's Return to Intel: M-Series Chip Deal by 2027</title>
	<url>https://apple.gadgethacks.com/news/apple-returns-to-intel-m-series-chip-deal-by-2027/</url>
	<favicon>https://assets.content.technologyadvice.com/gadgethacks_favicon_d352b3f01c.webp</favicon>
</citation>`;

		const result = parseCitation(input);

		expect(result.title).toBe("Apple's Return to Intel: M-Series Chip Deal by 2027");
		expect(result.url).toBe(
			'https://apple.gadgethacks.com/news/apple-returns-to-intel-m-series-chip-deal-by-2027/'
		);
		expect(result.favicon).toBe(
			'https://assets.content.technologyadvice.com/gadgethacks_favicon_d352b3f01c.webp'
		);
		expect(isCitationBlock(input)).toBe(true);
	});
});

describe('splitCitations', () => {
	it('should split consecutive citations', () => {
		const input = `<citation>
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

		const citations = splitCitations(input, 0);

		expect(citations).toHaveLength(3);

		// First citation
		expect(citations[0].text).toContain('Intel Makes U‑turn');
		expect(citations[0].from).toBe(0);
		expect(parseCitation(citations[0].text).title).toBe(
			"Intel Makes U‑turn, Cancels Plan to Sell Its Networking Division: Read Chipmaker's Statement"
		);
		expect(parseCitation(citations[0].text).url).toBe(
			'https://timesofindia.indiatimes.com/technology/tech-news/intel-makes-u-turn-cancels-plan-to-sell-its-networking-division-read-chipmakers-statement/articleshow/125769743.cms'
		);

		// Second citation
		expect(citations[1].text).toContain("Apple's Return to Intel");
		expect(parseCitation(citations[1].text).title).toBe(
			"Apple's Return to Intel: M-Series Chip Deal by 2027"
		);
		expect(parseCitation(citations[1].text).url).toBe(
			'https://apple.gadgethacks.com/news/apple-returns-to-intel-m-series-chip-deal-by-2027/'
		);

		// Third citation
		expect(citations[2].text).toContain('The 10 Biggest Intel News Stories Of 2025');
		expect(parseCitation(citations[2].text).title).toBe(
			'The 10 Biggest Intel News Stories Of 2025'
		);
		expect(parseCitation(citations[2].text).url).toBe(
			'https://www.crn.com/news/components-peripherals/2025/the-10-biggest-intel-news-stories-of-2025'
		);
	});
});
