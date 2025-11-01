import { describe, it, expect } from 'vitest';
import { parseCitation, isCitationBlock } from './citation-parser';

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
