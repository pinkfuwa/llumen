import { describe, test, expect } from 'vitest';
import { parse } from '../parser';
import { TokenType } from '../tokens';
import type {
	ParagraphToken,
	BoldToken,
	ItalicToken,
	StrikethroughToken,
	LinkToken,
	ImageToken,
	InlineCodeToken,
	TextToken,
	LineBreakToken
} from '../tokens';

describe('MarkdownParser - Inline Tokens', () => {
	describe('Bold', () => {
		test('parses bold with double asterisks', () => {
			const markdown = 'This is **bold** text.';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const boldTokens = para.children?.filter((t) => t.type === TokenType.Bold) || [];
			expect(boldTokens.length).toBe(1);
		});

		test('parses bold with double underscores', () => {
			const markdown = 'This is __bold__ text.';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const boldTokens = para.children?.filter((t) => t.type === TokenType.Bold) || [];
			expect(boldTokens.length).toBe(1);
		});

		test('parses multiple bold sections', () => {
			const markdown = '**First** and **second** bold.';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const boldTokens = para.children?.filter((t) => t.type === TokenType.Bold) || [];
			expect(boldTokens.length).toBe(2);
		});

		test('parses nested formatting in bold', () => {
			const markdown = '**Bold with *italic* inside**';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const boldToken = para.children?.find((t) => t.type === TokenType.Bold) as BoldToken;
			expect(boldToken).toBeDefined();
			expect(boldToken.children.length).toBeGreaterThan(1);
		});

		test('does not parse unclosed bold', () => {
			const markdown = '**Not closed';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const boldTokens = para.children?.filter((t) => t.type === TokenType.Bold) || [];
			expect(boldTokens.length).toBe(0);
		});
	});

	describe('Italic', () => {
		test('parses italic with single asterisks', () => {
			const markdown = 'This is *italic* text.';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const italicTokens = para.children?.filter((t) => t.type === TokenType.Italic) || [];
			expect(italicTokens.length).toBe(1);
		});

		test('parses italic with single underscores', () => {
			const markdown = 'This is _italic_ text.';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const italicTokens = para.children?.filter((t) => t.type === TokenType.Italic) || [];
			expect(italicTokens.length).toBe(1);
		});

		test('parses multiple italic sections', () => {
			const markdown = '*First* and *second* italic.';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const italicTokens = para.children?.filter((t) => t.type === TokenType.Italic) || [];
			expect(italicTokens.length).toBe(2);
		});

		test('parses bold and italic together', () => {
			const markdown = '**bold** and *italic*';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const boldTokens = para.children?.filter((t) => t.type === TokenType.Bold) || [];
			const italicTokens = para.children?.filter((t) => t.type === TokenType.Italic) || [];
			expect(boldTokens.length).toBe(1);
			expect(italicTokens.length).toBe(1);
		});

		test('does not parse unclosed italic', () => {
			const markdown = '*Not closed';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const italicTokens = para.children?.filter((t) => t.type === TokenType.Italic) || [];
			expect(italicTokens.length).toBe(0);
		});
	});

	describe('Strikethrough', () => {
		test('parses strikethrough', () => {
			const markdown = 'This is ~~deleted~~ text.';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const strikeTokens = para.children?.filter((t) => t.type === TokenType.Strikethrough) || [];
			expect(strikeTokens.length).toBe(1);
		});

		test('parses multiple strikethrough sections', () => {
			const markdown = '~~First~~ and ~~second~~ deleted.';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const strikeTokens = para.children?.filter((t) => t.type === TokenType.Strikethrough) || [];
			expect(strikeTokens.length).toBe(2);
		});

		test('parses nested formatting in strikethrough', () => {
			const markdown = '~~Strike with **bold** inside~~';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const strikeToken = para.children?.find(
				(t) => t.type === TokenType.Strikethrough
			) as StrikethroughToken;
			expect(strikeToken).toBeDefined();
			expect(strikeToken.children.length).toBeGreaterThan(1);
		});

		test('does not parse unclosed strikethrough', () => {
			const markdown = '~~Not closed';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const strikeTokens = para.children?.filter((t) => t.type === TokenType.Strikethrough) || [];
			expect(strikeTokens.length).toBe(0);
		});
	});

	describe('Inline Code', () => {
		test('parses inline code', () => {
			const markdown = 'This is `code` inline.';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const codeTokens = para.children?.filter((t) => t.type === TokenType.InlineCode) || [];
			expect(codeTokens.length).toBe(1);
			const codeToken = codeTokens[0] as InlineCodeToken;
			expect(codeToken.content).toBe('code');
		});

		test('parses multiple inline code sections', () => {
			const markdown = 'Code `first` and `second`.';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const codeTokens = para.children?.filter((t) => t.type === TokenType.InlineCode) || [];
			expect(codeTokens.length).toBe(2);
		});

		test('preserves spaces in inline code', () => {
			const markdown = 'Code with `  spaces  ` preserved.';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const codeToken = para.children?.find(
				(t) => t.type === TokenType.InlineCode
			) as InlineCodeToken;
			expect(codeToken.content).toBe('  spaces  ');
		});

		test('does not parse formatting inside inline code', () => {
			const markdown = 'Code `with **bold**` ignored.';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const codeToken = para.children?.find(
				(t) => t.type === TokenType.InlineCode
			) as InlineCodeToken;
			expect(codeToken.content).toBe('with **bold**');
		});

		test('does not parse unclosed inline code', () => {
			const markdown = 'Code `not closed';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const codeTokens = para.children?.filter((t) => t.type === TokenType.InlineCode) || [];
			expect(codeTokens.length).toBe(0);
		});
	});

	describe('Links', () => {
		test('parses simple link', () => {
			const markdown = '[Link text](https://example.com)';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const linkTokens = para.children?.filter((t) => t.type === TokenType.Link) || [];
			expect(linkTokens.length).toBe(1);
			const link = linkTokens[0] as LinkToken;
			expect(link.url).toBe('https://example.com');
		});

		test('parses link with title', () => {
			const markdown = '[Link](https://example.com "Title")';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const link = para.children?.find((t) => t.type === TokenType.Link) as LinkToken;
			expect(link.url).toBe('https://example.com "Title"');
		});

		test('parses multiple links', () => {
			const markdown = '[First](https://one.com) and [Second](https://two.com)';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const linkTokens = para.children?.filter((t) => t.type === TokenType.Link) || [];
			expect(linkTokens.length).toBe(2);
		});

		test('parses link with formatted text', () => {
			const markdown = '[**Bold** link](https://example.com)';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const link = para.children?.find((t) => t.type === TokenType.Link) as LinkToken;
			expect(link.children.length).toBeGreaterThan(0);
		});

		test('parses link with special characters in URL', () => {
			const markdown = '[Link](https://example.com/path?query=value&foo=bar#anchor)';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const link = para.children?.find((t) => t.type === TokenType.Link) as LinkToken;
			expect(link.url).toBe('https://example.com/path?query=value&foo=bar#anchor');
		});

		test('does not parse incomplete link', () => {
			const markdown = '[Text only';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const linkTokens = para.children?.filter((t) => t.type === TokenType.Link) || [];
			expect(linkTokens.length).toBe(0);
		});

		test('parses angle-bracketed URL as link', () => {
			const markdown = '<https://datatracker.ietf.org/doc/html/rfc6238>';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const linkTokens = para.children?.filter((t) => t.type === TokenType.Link) || [];
			expect(linkTokens.length).toBe(1);
			const link = linkTokens[0] as LinkToken;
			expect(link.url).toBe('https://datatracker.ietf.org/doc/html/rfc6238');
			// The link text should be the URL itself
			const textToken = link.children?.[0] as TextToken;
			expect(textToken.type).toBe(TokenType.Text);
			expect(textToken.content).toBe('https://datatracker.ietf.org/doc/html/rfc6238');
		});

		test('parses multiple angle-bracketed URLs', () => {
			const markdown = 'Visit <https://example.com> and <https://test.org>';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const linkTokens = para.children?.filter((t) => t.type === TokenType.Link) || [];
			expect(linkTokens.length).toBe(2);
		});

		test('parses angle-bracketed URL with path and query', () => {
			const markdown = '<https://example.com/path?query=value&foo=bar#anchor>';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const link = para.children?.find((t) => t.type === TokenType.Link) as LinkToken;
			expect(link.url).toBe('https://example.com/path?query=value&foo=bar#anchor');
		});

		test('parses angle-bracketed HTTP URL (not just HTTPS)', () => {
			const markdown = '<http://example.com>';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const link = para.children?.find((t) => t.type === TokenType.Link) as LinkToken;
			expect(link).toBeDefined();
			expect(link.url).toBe('http://example.com');
		});
	});

	describe('Images', () => {
		test('parses simple image', () => {
			const markdown = '![Alt text](https://example.com/image.png)';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const imageTokens = para.children?.filter((t) => t.type === TokenType.Image) || [];
			expect(imageTokens.length).toBe(1);
			const image = imageTokens[0] as ImageToken;
			expect(image.url).toBe('https://example.com/image.png');
			expect(image.alt).toBe('Alt text');
		});

		test('parses image with title', () => {
			const markdown = '![Alt](https://example.com/img.jpg "Image Title")';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const image = para.children?.find((t) => t.type === TokenType.Image) as ImageToken;
			expect(image.url).toBe('https://example.com/img.jpg "Image Title"');
			expect(image.alt).toBe('Alt');
		});

		test('parses multiple images', () => {
			const markdown = '![First](img1.png) and ![Second](img2.png)';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const imageTokens = para.children?.filter((t) => t.type === TokenType.Image) || [];
			expect(imageTokens.length).toBe(2);
		});

		test('parses image with empty alt text', () => {
			const markdown = '![](https://example.com/image.png)';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const image = para.children?.find((t) => t.type === TokenType.Image) as ImageToken;
			expect(image.alt).toBe('');
			expect(image.url).toBe('https://example.com/image.png');
		});

		test('does not parse incomplete image', () => {
			const markdown = '![Alt only';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const imageTokens = para.children?.filter((t) => t.type === TokenType.Image) || [];
			expect(imageTokens.length).toBe(0);
		});
	});

	describe('Line Breaks', () => {
		test('parses <br> tag as line break', () => {
			const markdown = 'Line one<br>Line two';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const lineBreakTokens = para.children?.filter((t) => t.type === TokenType.LineBreak) || [];
			expect(lineBreakTokens.length).toBe(1);
		});

		test('parses <br /> tag as line break', () => {
			const markdown = 'Line one<br />Line two';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const lineBreakTokens = para.children?.filter((t) => t.type === TokenType.LineBreak) || [];
			expect(lineBreakTokens.length).toBe(1);
		});

		test('parses <br/> tag (without space) as line break', () => {
			const markdown = 'Line one<br/>Line two';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const lineBreakTokens = para.children?.filter((t) => t.type === TokenType.LineBreak) || [];
			expect(lineBreakTokens.length).toBe(1);
		});

		test('parses multiple <br> tags', () => {
			const markdown = 'Line one<br>Line two<br>Line three';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const lineBreakTokens = para.children?.filter((t) => t.type === TokenType.LineBreak) || [];
			expect(lineBreakTokens.length).toBe(2);
		});

		test('parses mixed <br> and <br /> tags', () => {
			const markdown = 'Line one<br>Line two<br />Line three<br/>Line four';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const lineBreakTokens = para.children?.filter((t) => t.type === TokenType.LineBreak) || [];
			expect(lineBreakTokens.length).toBe(3);
		});

		test('parses two spaces followed by newline as line break', () => {
			const markdown = 'Line one  \nLine two';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const lineBreakTokens = para.children?.filter((t) => t.type === TokenType.LineBreak) || [];
			expect(lineBreakTokens.length).toBe(1);
		});

		test('parses <br> with surrounding text', () => {
			const markdown = 'Text before<br>text after with **bold**';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const lineBreakTokens = para.children?.filter((t) => t.type === TokenType.LineBreak) || [];
			expect(lineBreakTokens.length).toBe(1);
			expect(para.children?.some((t) => t.type === TokenType.Bold)).toBe(true);
		});
	});

	describe('Mixed inline formatting', () => {
		test('parses bold and italic together', () => {
			const markdown = '**bold** and *italic* text';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const boldTokens = para.children?.filter((t) => t.type === TokenType.Bold) || [];
			const italicTokens = para.children?.filter((t) => t.type === TokenType.Italic) || [];
			expect(boldTokens.length).toBe(1);
			expect(italicTokens.length).toBe(1);
		});

		test('parses complex inline combinations', () => {
			const markdown = 'Text with **bold**, *italic*, `code`, and [link](url).';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			expect(para.children?.some((t) => t.type === TokenType.Bold)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Italic)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.InlineCode)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Link)).toBe(true);
		});

		test('parses nested bold and italic', () => {
			const markdown = '**bold with *italic* inside**';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const boldToken = para.children?.find((t) => t.type === TokenType.Bold) as BoldToken;
			expect(boldToken.children.some((t) => t.type === TokenType.Italic)).toBe(true);
		});

		test('parses link with inline formatting', () => {
			const markdown = '[Link with **bold** and *italic*](https://example.com)';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			const link = para.children?.find((t) => t.type === TokenType.Link) as LinkToken;
			expect(link.children.some((t) => t.type === TokenType.Bold)).toBe(true);
			expect(link.children.some((t) => t.type === TokenType.Italic)).toBe(true);
		});

		test('parses all inline types in one paragraph', () => {
			const markdown =
				'**Bold**, *italic*, ~~strike~~, `code`, [link](url), ![img](img.png), and text.';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			expect(para.children?.some((t) => t.type === TokenType.Bold)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Italic)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Strikethrough)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.InlineCode)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Link)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Image)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Text)).toBe(true);
		});
	});

	describe('Edge cases', () => {
		test('handles empty formatting markers', () => {
			const markdown = 'Text with ** ** empty bold';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			expect(para.children).toBeDefined();
		});

		test('handles overlapping but not nested markers', () => {
			const markdown = '**bold *italic** still italic*';
			const result = parse(markdown);
			expect(result.tokens.length).toBe(1);
		});

		test('handles escaped characters', () => {
			const markdown = 'Text with \\*asterisks\\*';
			const result = parse(markdown);
			expect(result.tokens[0].type).toBe(TokenType.Paragraph);
		});

		test('parses consecutive formatting', () => {
			const markdown = '**bold***italic*';
			const result = parse(markdown);
			const para = result.tokens[0] as ParagraphToken;
			expect(para.children?.some((t) => t.type === TokenType.Bold)).toBe(true);
			expect(para.children?.some((t) => t.type === TokenType.Italic)).toBe(true);
		});
	});
});
