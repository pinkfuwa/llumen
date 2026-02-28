import { describe, test, expect } from 'vitest';
import { TokenType } from '../tokens';
import * as builders from '../tokens/builders';

describe('TokenBuilders', () => {
	describe('TextToken', () => {
		test('creates text token with correct properties', () => {
			const token = builders.createTextToken('hello', 0, 5);
			expect(token.type).toBe(TokenType.Text);
			expect(token.content).toBe('hello');
			expect(token.start).toBe(0);
			expect(token.end).toBe(5);
		});

		test('creates text token with empty content', () => {
			const token = builders.createTextToken('', 0, 0);
			expect(token.type).toBe(TokenType.Text);
			expect(token.content).toBe('');
		});

		test('creates text token with unicode content', () => {
			const token = builders.createTextToken('你好世界', 0, 4);
			expect(token.content).toBe('你好世界');
		});

		test('creates text token with special characters', () => {
			const token = builders.createTextToken('!@#$%^&*()', 0, 9);
			expect(token.content).toBe('!@#$%^&*()');
		});
	});

	describe('HeadingToken', () => {
		test('creates heading token with level 1', () => {
			const children: any[] = [];
			const token = builders.createHeadingToken(1, children, 0, 10);
			expect(token.type).toBe(TokenType.Heading);
			expect(token.level).toBe(1);
			expect(token.start).toBe(0);
			expect(token.end).toBe(10);
		});

		test('creates heading token with level 6', () => {
			const children: any[] = [];
			const token = builders.createHeadingToken(6, children, 0, 10);
			expect(token.level).toBe(6);
		});

		test('heading token includes children', () => {
			const childToken = builders.createTextToken('bold', 2, 6);
			const token = builders.createHeadingToken(1, [childToken], 0, 10);
			expect(token.children.length).toBe(1);
		});
	});

	describe('ParagraphToken', () => {
		test('creates paragraph token', () => {
			const children: any[] = [];
			const token = builders.createParagraphToken(children, 0, 10);
			expect(token.type).toBe(TokenType.Paragraph);
			expect(token.start).toBe(0);
			expect(token.end).toBe(10);
		});
	});

	describe('CodeBlockToken', () => {
		test('creates code block with language', () => {
			const token = builders.createCodeBlockToken('javascript', 'const x = 1;', true, 0, 30);
			expect(token.type).toBe(TokenType.CodeBlock);
			expect(token.language).toBe('javascript');
			expect(token.content).toBe('const x = 1;');
			expect(token.closed).toBe(true);
		});

		test('creates code block without language', () => {
			const token = builders.createCodeBlockToken(undefined, 'code', true, 0, 10);
			expect(token.language).toBeUndefined();
		});

		test('creates unclosed code block (streaming)', () => {
			const token = builders.createCodeBlockToken('python', 'print("hello")', false, 0, 20);
			expect(token.closed).toBe(false);
		});
	});

	describe('BlockquoteToken', () => {
		test('creates blockquote token', () => {
			const children: any[] = [];
			const token = builders.createBlockquoteToken(children, 0, 10);
			expect(token.type).toBe(TokenType.Blockquote);
		});

		test('blockquote includes nested tokens', () => {
			const childPara = builders.createParagraphToken([], 2, 8);
			const token = builders.createBlockquoteToken([childPara], 0, 10);
			expect(token.children.length).toBe(1);
		});
	});

	describe('OrderedListToken', () => {
		test('creates ordered list with start number', () => {
			const items: any[] = [];
			const token = builders.createOrderedListToken(5, items, 0, 20);
			expect(token.type).toBe(TokenType.OrderedList);
			expect(token.startNumber).toBe(5);
		});

		test('creates ordered list without start number', () => {
			const items: any[] = [];
			const token = builders.createOrderedListToken(undefined, items, 0, 20);
			expect(token.startNumber).toBeUndefined();
		});
	});

	describe('UnorderedListToken', () => {
		test('creates unordered list token', () => {
			const items: any[] = [];
			const token = builders.createUnorderedListToken(items, 0, 20);
			expect(token.type).toBe(TokenType.UnorderedList);
		});
	});

	describe('ListItemToken', () => {
		test('creates list item token', () => {
			const children: any[] = [];
			const token = builders.createListItemToken(children, 0, 10);
			expect(token.type).toBe(TokenType.ListItem);
		});
	});

	describe('TableToken', () => {
		test('creates table token', () => {
			const rows: any[] = [];
			const token = builders.createTableToken(rows, 0, 20);
			expect(token.type).toBe(TokenType.Table);
		});
	});

	describe('TableRowToken', () => {
		test('creates header row', () => {
			const cells: any[] = [];
			const token = builders.createTableRowToken(true, cells, 0, 10);
			expect(token.type).toBe(TokenType.TableRow);
			expect(token.isHeader).toBe(true);
		});

		test('creates data row', () => {
			const cells: any[] = [];
			const token = builders.createTableRowToken(false, cells, 0, 10);
			expect(token.isHeader).toBe(false);
		});
	});

	describe('TableCellToken', () => {
		test('creates cell with left alignment', () => {
			const children: any[] = [];
			const token = builders.createTableCellToken('left', children, 0, 5);
			expect(token.type).toBe(TokenType.TableCell);
			expect(token.align).toBe('left');
		});

		test('creates cell with center alignment', () => {
			const children: any[] = [];
			const token = builders.createTableCellToken('center', children, 0, 5);
			expect(token.align).toBe('center');
		});

		test('creates cell with right alignment', () => {
			const children: any[] = [];
			const token = builders.createTableCellToken('right', children, 0, 5);
			expect(token.align).toBe('right');
		});

		test('creates cell without alignment', () => {
			const children: any[] = [];
			const token = builders.createTableCellToken(undefined, children, 0, 5);
			expect(token.align).toBeUndefined();
		});
	});

	describe('HorizontalRuleToken', () => {
		test('creates horizontal rule token', () => {
			const token = builders.createHorizontalRuleToken(0, 3);
			expect(token.type).toBe(TokenType.HorizontalRule);
			expect(token.start).toBe(0);
			expect(token.end).toBe(3);
		});
	});

	describe('LatexBlockToken', () => {
		test('creates latex block token', () => {
			const token = builders.createLatexBlockToken('x^2 + y^2', 0, 15);
			expect(token.type).toBe(TokenType.LatexBlock);
			expect(token.content).toBe('x^2 + y^2');
		});

		test('creates latex block with complex content', () => {
			const token = builders.createLatexBlockToken('\\frac{a}{b}', 0, 12);
			expect(token.content).toBe('\\frac{a}{b}');
		});
	});

	describe('LatexInlineToken', () => {
		test('creates latex inline token', () => {
			const token = builders.createLatexInlineToken('x^2', 0, 5);
			expect(token.type).toBe(TokenType.LatexInline);
			expect(token.content).toBe('x^2');
		});
	});

	describe('BoldToken', () => {
		test('creates bold token', () => {
			const children: any[] = [];
			const token = builders.createBoldToken(children, 0, 10);
			expect(token.type).toBe(TokenType.Bold);
		});
	});

	describe('ItalicToken', () => {
		test('creates italic token', () => {
			const children: any[] = [];
			const token = builders.createItalicToken(children, 0, 8);
			expect(token.type).toBe(TokenType.Italic);
		});
	});

	describe('StrikethroughToken', () => {
		test('creates strikethrough token', () => {
			const children: any[] = [];
			const token = builders.createStrikethroughToken(children, 0, 12);
			expect(token.type).toBe(TokenType.Strikethrough);
		});
	});

	describe('InlineCodeToken', () => {
		test('creates inline code token', () => {
			const token = builders.createInlineCodeToken('const x = 1', 0, 15);
			expect(token.type).toBe(TokenType.InlineCode);
			expect(token.content).toBe('const x = 1');
		});

		test('creates inline code with special chars', () => {
			const token = builders.createInlineCodeToken('<div></div>', 0, 12);
			expect(token.content).toBe('<div></div>');
		});
	});

	describe('LinkToken', () => {
		test('creates link token', () => {
			const children: any[] = [];
			const token = builders.createLinkToken('https://example.com', children, 0, 25);
			expect(token.type).toBe(TokenType.Link);
			expect(token.url).toBe('https://example.com');
		});
	});

	describe('ImageToken', () => {
		test('creates image token', () => {
			const token = builders.createImageToken('https://example.com/img.png', 'Alt text', 0, 35);
			expect(token.type).toBe(TokenType.Image);
			expect(token.url).toBe('https://example.com/img.png');
			expect(token.alt).toBe('Alt text');
		});

		test('creates image with empty alt', () => {
			const token = builders.createImageToken('https://example.com/img.png', '', 0, 25);
			expect(token.alt).toBe('');
		});
	});

	describe('LineBreakToken', () => {
		test('creates line break token', () => {
			const token = builders.createLineBreakToken(0, 1);
			expect(token.type).toBe(TokenType.LineBreak);
			expect(token.start).toBe(0);
			expect(token.end).toBe(1);
		});
	});

	describe('Position Tracking', () => {
		test('all tokens track positions correctly', () => {
			const start = 10;
			const end = 20;
			const delta = end - start;

			const text = builders.createTextToken('test', start, end);
			const heading = builders.createHeadingToken(1, [], start, end);
			const paragraph = builders.createParagraphToken([], start, end);
			const codeBlock = builders.createCodeBlockToken('js', 'code', true, start, end);
			const blockquote = builders.createBlockquoteToken([], start, end);
			const orderedList = builders.createOrderedListToken(1, [], start, end);
			const unorderedList = builders.createUnorderedListToken([], start, end);
			const listItem = builders.createListItemToken([], start, end);
			const table = builders.createTableToken([], start, end);
			const tableRow = builders.createTableRowToken(true, [], start, end);
			const tableCell = builders.createTableCellToken(undefined, [], start, end);
			const hr = builders.createHorizontalRuleToken(start, end);
			const latexBlock = builders.createLatexBlockToken('x', start, end);
			const latexInline = builders.createLatexInlineToken('x', start, end);
			const bold = builders.createBoldToken([], start, end);
			const italic = builders.createItalicToken([], start, end);
			const strike = builders.createStrikethroughToken([], start, end);
			const inlineCode = builders.createInlineCodeToken('x', start, end);
			const link = builders.createLinkToken('url', [], start, end);
			const image = builders.createImageToken('url', 'alt', start, end);
			const lineBreak = builders.createLineBreakToken(start, end);

			const tokens = [
				text,
				heading,
				paragraph,
				codeBlock,
				blockquote,
				orderedList,
				unorderedList,
				listItem,
				table,
				tableRow,
				tableCell,
				hr,
				latexBlock,
				latexInline,
				bold,
				italic,
				strike,
				inlineCode,
				link,
				image,
				lineBreak
			];

			tokens.forEach((token) => {
				expect(token.start).toBe(start);
				expect(token.end).toBe(end);
			});
		});
	});
});
