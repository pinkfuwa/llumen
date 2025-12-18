import { describe, test, expect } from 'vitest';
import { parse } from '../parser';
import { TokenType } from '../tokens';
import type { CitationToken, ParagraphToken } from '../tokens';

describe('MarkdownParser - Citation Tokens', () => {
	describe('Inline citation format [@cite:id]', () => {
		test('parses simple inline citation', () => {
			const markdown = 'This is a claim [@cite:123].';
			const result = parse(markdown);

			expect(result.tokens.length).toBe(1);
			const para = result.tokens[0] as ParagraphToken;
			expect(para.type).toBe(TokenType.Paragraph);

			const citationTokens = para.children?.filter((t) => t.type === TokenType.Citation) || [];
			expect(citationTokens.length).toBe(1);

			const citation = citationTokens[0] as CitationToken;
			expect(citation.id).toBe('cite:123');
			expect(citation.title).toBeUndefined();
			expect(citation.url).toBeUndefined();
			expect(citation.favicon).toBeUndefined();
			expect(citation.authoritative).toBe(false);
		});

		test('parses multiple inline citations', () => {
			const markdown = 'First claim [@cite:abc] and second [@cite:xyz].';
			const result = parse(markdown);

			expect(result.tokens.length).toBe(1);
			const para = result.tokens[0] as ParagraphToken;
			const citationTokens = para.children?.filter((t) => t.type === TokenType.Citation) || [];
			expect(citationTokens.length).toBe(2);

			expect((citationTokens[0] as CitationToken).id).toBe('cite:abc');
			expect((citationTokens[1] as CitationToken).id).toBe('cite:xyz');
		});

		test('parses citation with special characters in id', () => {
			const markdown = 'Reference [@cite:foo-bar_123].';
			const result = parse(markdown);

			const para = result.tokens[0] as ParagraphToken;
			const citationTokens = para.children?.filter((t) => t.type === TokenType.Citation) || [];
			expect(citationTokens.length).toBe(1);
			expect((citationTokens[0] as CitationToken).id).toBe('cite:foo-bar_123');
		});
	});

	describe('Block citation format <citation>', () => {
		test('parses citation with all fields', () => {
			const markdown = `<citation>
    <title>澳門美食攻略：蛋撻、豬扒包、葡國料理推薦</title>
    <url>https://www.macaotourism.gov.mo/zh-hant/dining/feature-macau-cuisine</url>
    <favicon>https://www.macaotourism.gov.mo/favicon.ico</favicon>
    <authoritative/>
</citation>`;

			const result = parse(markdown);
			expect(result.tokens.length).toBe(1);

			const citation = result.tokens[0] as CitationToken;
			expect(citation.type).toBe(TokenType.Citation);
			expect(citation.title).toBe('澳門美食攻略：蛋撻、豬扒包、葡國料理推薦');
			expect(citation.url).toBe(
				'https://www.macaotourism.gov.mo/zh-hant/dining/feature-macau-cuisine'
			);
			expect(citation.favicon).toBe('https://www.macaotourism.gov.mo/favicon.ico');
			expect(citation.authoritative).toBe(true);
		});

		test('parses citation without authoritative tag (defaults to false)', () => {
			const markdown = `<citation>
    <title>澳門十大必吃美食及推薦餐廳</title>
    <url>https://travel.ulifestyle.com.hk/news/detail/20009939</url>
    <favicon>https://travel.ulifestyle.com.hk/favicon.ico</favicon>
</citation>`;

			const result = parse(markdown);
			expect(result.tokens.length).toBe(1);

			const citation = result.tokens[0] as CitationToken;
			expect(citation.type).toBe(TokenType.Citation);
			expect(citation.title).toBe('澳門十大必吃美食及推薦餐廳');
			expect(citation.authoritative).toBe(false);
		});

		test('parses citation with self-closing authoritative tag with space', () => {
			const markdown = `<citation>
    <title>Test Title</title>
    <url>https://example.com</url>
    <authoritative />
</citation>`;

			const result = parse(markdown);
			const citation = result.tokens[0] as CitationToken;
			expect(citation.authoritative).toBe(true);
		});

		test('parses citation with paired authoritative tag', () => {
			const markdown = `<citation>
    <title>Test Title</title>
    <url>https://example.com</url>
    <authoritative></authoritative>
</citation>`;

			const result = parse(markdown);
			const citation = result.tokens[0] as CitationToken;
			expect(citation.authoritative).toBe(true);
		});

		test('parses citation with only title and url', () => {
			const markdown = `<citation>
    <title>Minimal Citation</title>
    <url>https://example.com</url>
</citation>`;

			const result = parse(markdown);
			const citation = result.tokens[0] as CitationToken;
			expect(citation.title).toBe('Minimal Citation');
			expect(citation.url).toBe('https://example.com');
			expect(citation.favicon).toBeUndefined();
			expect(citation.authoritative).toBe(false);
		});

		test('parses citation with only title', () => {
			const markdown = `<citation>
    <title>Title Only Citation</title>
</citation>`;

			const result = parse(markdown);
			const citation = result.tokens[0] as CitationToken;
			expect(citation.title).toBe('Title Only Citation');
			expect(citation.url).toBeUndefined();
			expect(citation.favicon).toBeUndefined();
			expect(citation.id).toBe('Title Only Citation'); // Falls back to title as id
		});

		test('parses multiple consecutive citations', () => {
			const markdown = `<citation>
    <title>First Citation</title>
    <url>https://first.com</url>
    <favicon>https://first.com/favicon.ico</favicon>
    <authoritative/>
</citation>
<citation>
    <title>Second Citation</title>
    <url>https://second.com</url>
    <favicon>https://second.com/favicon.ico</favicon>
</citation>
<citation>
    <title>Third Citation</title>
    <url>https://third.com</url>
    <favicon>https://third.com/favicon.ico</favicon>
    <authoritative/>
</citation>`;

			const result = parse(markdown);
			expect(result.tokens.length).toBe(3);

			const citation1 = result.tokens[0] as CitationToken;
			expect(citation1.title).toBe('First Citation');
			expect(citation1.authoritative).toBe(true);

			const citation2 = result.tokens[1] as CitationToken;
			expect(citation2.title).toBe('Second Citation');
			expect(citation2.authoritative).toBe(false);

			const citation3 = result.tokens[2] as CitationToken;
			expect(citation3.title).toBe('Third Citation');
			expect(citation3.authoritative).toBe(true);
		});

		test('parses citation with special characters in title', () => {
			const markdown = `<citation>
    <title>Title with "quotes" & special <chars></title>
    <url>https://example.com</url>
</citation>`;

			const result = parse(markdown);
			const citation = result.tokens[0] as CitationToken;
			expect(citation.title).toBe('Title with "quotes" & special <chars>');
		});

		test('parses citation with multiline content', () => {
			const markdown = `<citation>
    <title>This is a very long title
    that spans multiple lines
    for testing purposes</title>
    <url>https://example.com/very/long/url/path</url>
</citation>`;

			const result = parse(markdown);
			const citation = result.tokens[0] as CitationToken;
			expect(citation.title).toContain('This is a very long title');
			expect(citation.title).toContain('that spans multiple lines');
		});

		test('handles citation without closing tag gracefully', () => {
			const markdown = `<citation>
    <title>Incomplete Citation</title>
    <url>https://example.com</url>`;

			const result = parse(markdown);
			// Should not parse as citation if closing tag is missing
			expect(result.tokens[0]?.type).not.toBe(TokenType.Citation);
		});

		test('parses citation with compact format', () => {
			const markdown =
				'<citation><title>Compact</title><url>https://example.com</url><authoritative/></citation>';

			const result = parse(markdown);
			const citation = result.tokens[0] as CitationToken;
			expect(citation.title).toBe('Compact');
			expect(citation.url).toBe('https://example.com');
			expect(citation.authoritative).toBe(true);
		});

		test('parses citation with encoded URL', () => {
			const markdown = `<citation>
    <title>URL with encoding</title>
    <url>https://example.com/path?query=value%20with%20spaces&foo=bar</url>
</citation>`;

			const result = parse(markdown);
			const citation = result.tokens[0] as CitationToken;
			expect(citation.url).toBe('https://example.com/path?query=value%20with%20spaces&foo=bar');
		});
	});

	describe('Mixed content with citations', () => {
		test('parses paragraph followed by block citation', () => {
			const markdown = `This is some text with inline citation [@cite:1].

<citation>
    <title>Block Citation</title>
    <url>https://example.com</url>
</citation>`;

			const result = parse(markdown);
			expect(result.tokens.length).toBe(2);
			expect(result.tokens[0].type).toBe(TokenType.Paragraph);
			expect(result.tokens[1].type).toBe(TokenType.Citation);
		});

		test('parses block citation followed by paragraph', () => {
			const markdown = `<citation>
    <title>Block Citation</title>
    <url>https://example.com</url>
</citation>

This is text after the citation.`;

			const result = parse(markdown);
			expect(result.tokens.length).toBe(2);
			expect(result.tokens[0].type).toBe(TokenType.Citation);
			expect(result.tokens[1].type).toBe(TokenType.Paragraph);
		});

		test('parses heading, citation, and paragraph', () => {
			const markdown = `# Heading

<citation>
    <title>Citation Title</title>
    <url>https://example.com</url>
</citation>

Regular paragraph text.`;

			const result = parse(markdown);
			expect(result.tokens.length).toBe(3);
			expect(result.tokens[0].type).toBe(TokenType.Heading);
			expect(result.tokens[1].type).toBe(TokenType.Citation);
			expect(result.tokens[2].type).toBe(TokenType.Paragraph);
		});
	});

	describe('Edge cases', () => {
		test('does not parse incomplete citation tag', () => {
			const markdown = '<citation>';
			const result = parse(markdown);
			// Should parse as paragraph with text, not as citation
			expect(result.tokens[0]?.type).not.toBe(TokenType.Citation);
		});

		test('handles empty citation block', () => {
			const markdown = '<citation></citation>';
			const result = parse(markdown);
			const citation = result.tokens[0] as CitationToken;
			expect(citation.type).toBe(TokenType.Citation);
			expect(citation.title).toBeUndefined();
			expect(citation.url).toBeUndefined();
		});

		test('parses citation with whitespace variations', () => {
			const markdown = `<citation>
			<title>   Title with spaces   </title>
			<url>   https://example.com   </url>
</citation>`;

			const result = parse(markdown);
			const citation = result.tokens[0] as CitationToken;
			expect(citation.title).toBe('Title with spaces');
			expect(citation.url).toBe('https://example.com');
		});
	});

	describe('User-provided examples', () => {
		test('parses the exact user-provided markdown with three citations', () => {
			const markdown = `<citation>
    <title>澳門美食攻略：蛋撻、豬扒包、葡國料理推薦</title>
    <url>https://www.macaotourism.gov.mo/zh-hant/dining/feature-macau-cuisine</url>
    <favicon>https://www.macaotourism.gov.mo/favicon.ico</favicon>
    <authoritative/>
</citation>
<citation>
    <title>澳門十大必吃美食及推薦餐廳</title>
    <url>https://travel.ulifestyle.com.hk/news/detail/20009939/%E6%BE%B3%E9%96%80%E5%BF%85%E5%90%83-%E5%BE%B7%E6%99%BA%E6%BF%BE%E6%BE%B3%E9%96%80%E7%BE%8E%E9%A3%9F-%E8%9B%87%E7%9A%AE%E9%A4%A8-%E8%B1%AC%E6%89%92%E5%8C%85</url>
    <favicon>https://travel.ulifestyle.com.hk/favicon.ico</favicon>
</citation>
<citation>
    <title>澳門旅遊局：澳門經典美食推薦</title>
    <url>https://www.macaotourism.gov.mo/zh-hant/dining/food-culture</url>
    <favicon>https://www.macaotourism.gov.mo/favicon.ico</favicon>
    <authoritative/>
</citation>`;

			const result = parse(markdown);
			expect(result.tokens.length).toBe(3);

			// First citation - authoritative
			const citation1 = result.tokens[0] as CitationToken;
			expect(citation1.type).toBe(TokenType.Citation);
			expect(citation1.title).toBe('澳門美食攻略：蛋撻、豬扒包、葡國料理推薦');
			expect(citation1.url).toBe(
				'https://www.macaotourism.gov.mo/zh-hant/dining/feature-macau-cuisine'
			);
			expect(citation1.favicon).toBe('https://www.macaotourism.gov.mo/favicon.ico');
			expect(citation1.authoritative).toBe(true);

			// Second citation - not authoritative (no tag)
			const citation2 = result.tokens[1] as CitationToken;
			expect(citation2.type).toBe(TokenType.Citation);
			expect(citation2.title).toBe('澳門十大必吃美食及推薦餐廳');
			expect(citation2.url).toBe(
				'https://travel.ulifestyle.com.hk/news/detail/20009939/%E6%BE%B3%E9%96%80%E5%BF%85%E5%90%83-%E5%BE%B7%E6%99%BA%E6%BF%BE%E6%BE%B3%E9%96%80%E7%BE%8E%E9%A3%9F-%E8%9B%87%E7%9A%AE%E9%A4%A8-%E8%B1%AC%E6%89%92%E5%8C%85'
			);
			expect(citation2.favicon).toBe('https://travel.ulifestyle.com.hk/favicon.ico');
			expect(citation2.authoritative).toBe(false); // Should default to false

			// Third citation - authoritative
			const citation3 = result.tokens[2] as CitationToken;
			expect(citation3.type).toBe(TokenType.Citation);
			expect(citation3.title).toBe('澳門旅遊局：澳門經典美食推薦');
			expect(citation3.url).toBe('https://www.macaotourism.gov.mo/zh-hant/dining/food-culture');
			expect(citation3.favicon).toBe('https://www.macaotourism.gov.mo/favicon.ico');
			expect(citation3.authoritative).toBe(true);
		});
	});
});
