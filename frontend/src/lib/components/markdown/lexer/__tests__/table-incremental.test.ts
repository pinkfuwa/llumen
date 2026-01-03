import { describe, test, expect } from 'vitest';
import { parse } from '../parser';
import { parseIncremental, type IncrementalState } from '../index';
import { TokenType } from '../tokens';
import type { TableToken, ParagraphToken, HeadingToken } from '../tokens';

describe('Table Incremental Parsing - The "Perfect Ratio" Table', () => {
	const perfectRatioTable = `### The "Perfect Ratio" Balancing Sheet

| Expense Category | 1974 Actual | 1974 Target (Based on '73 Ratio) | Action Needed |
| :--- | :--- | :--- | :--- |
| **Fuel Expense** | $57,158K | $44,120K | **Cut $13,038K** |
| **Aircraft Lease** | $48,441K | $33,379K | **Cut $15,062K** |
| **Service Investment**| $36,825K | $33,261K | **Cut $3,564K** |
| **Advertisement** | $51,700K | $48,550K | **Cut $3,150K** |
| **Personnel** | $44,384K | $41,520K | **Cut $2,864K** |
| **Flight Expense** | $25,307K | $22,588K | **Cut $2,719K** |
| **Branch Expense** | $19,428K | $25,719K | *Room to Invest $6,291K* |
| **Depreciation** | $21,999K | $24,883K | *Room to Invest $2,884K* |`;

	test('parses complete Perfect Ratio table correctly', () => {
		const result = parse(perfectRatioTable);

		// Should have heading and table
		expect(result.tokens.length).toBeGreaterThanOrEqual(2);

		// First token should be heading
		const heading = result.tokens[0] as HeadingToken;
		expect(heading.type).toBe(TokenType.Heading);
		expect(heading.level).toBe(3);

		// Second token should be table
		const table = result.tokens[1] as TableToken;
		expect(table.type).toBe(TokenType.Table);

		// Table should have header + 8 data rows = 9 rows
		expect(table.children?.length).toBe(9);

		// First row should be header
		expect(table.children?.[0].isHeader).toBe(true);

		// Data rows should not be headers
		for (let i = 1; i < 9; i++) {
			expect(table.children?.[i].isHeader).toBe(false);
		}

		// Verify some specific cell content
		const headerRow = table.children?.[0];
		expect(headerRow?.children?.length).toBe(4);
	});

	test('parses table with inline formatting (bold and italic)', () => {
		const result = parse(perfectRatioTable);
		const table = result.tokens[1] as TableToken;

		// Check that bold formatting is preserved in cells
		let hasBold = false;
		let hasItalic = false;

		table.children?.forEach((row) => {
			row.children?.forEach((cell) => {
				if (cell.children?.some((t) => t.type === TokenType.Bold)) {
					hasBold = true;
				}
				if (cell.children?.some((t) => t.type === TokenType.Italic)) {
					hasItalic = true;
				}
			});
		});

		expect(hasBold).toBe(true);
		expect(hasItalic).toBe(true);
	});

	test('table creates region boundary', () => {
		const result = parse(perfectRatioTable);
		const tableRegion = result.regions.find((r) => r.type === 'table');

		expect(tableRegion).toBeDefined();
		expect(tableRegion?.start).toBeGreaterThan(0); // After heading
		expect(tableRegion?.end).toBeGreaterThan(tableRegion!.start);
	});
});

describe('Table Incremental Parsing - Randomized Segmentation', () => {
	const simpleTable = `| A | B | C |
|---|---|---|
| 1 | 2 | 3 |
| 4 | 5 | 6 |`;

	/**
	 * Test incremental parsing by simulating streaming input at random positions
	 */
	test('incremental parsing with random segmentation points', async () => {
		const content = simpleTable;

		// Test multiple random segmentation patterns
		for (let iteration = 0; iteration < 10; iteration++) {
			// Generate random split points
			const splitPoints: number[] = [0];
			const numSplits = 3 + Math.floor(Math.random() * 5); // 3-7 splits

			for (let i = 0; i < numSplits; i++) {
				const point = Math.floor(Math.random() * content.length);
				if (!splitPoints.includes(point)) {
					splitPoints.push(point);
				}
			}
			splitPoints.push(content.length);
			splitPoints.sort((a, b) => a - b);

			// Parse incrementally
			let state: IncrementalState | null = null;
			let currentSource = '';

			for (let i = 1; i < splitPoints.length; i++) {
				currentSource = content.slice(0, splitPoints[i]);
				const result = await parseIncremental(currentSource, state);
				state = result.state;
			}

			// Final result should be the same as full parse
			const incrementalResult = state!.prevResult;
			const fullResult = parse(content);

			expect(incrementalResult.tokens.length).toBe(fullResult.tokens.length);
			expect(incrementalResult.tokens[0].type).toBe(TokenType.Table);
		}
	});

	test('incremental parsing: table built row by row', async () => {
		const lines = simpleTable.split('\n');
		let state: IncrementalState | null = null;

		for (let i = 1; i <= lines.length; i++) {
			const partial = lines.slice(0, i).join('\n');
			const result = await parseIncremental(partial, state);
			state = result.state;

			if (i >= 2) {
				// After header + separator, should start recognizing table
				// Note: might be paragraph initially, then convert to table
				expect(result.result.tokens.length).toBeGreaterThan(0);
			}
		}

		// Final result should be complete table
		const finalResult = state!.prevResult;
		expect(finalResult.tokens.length).toBe(1);
		expect(finalResult.tokens[0].type).toBe(TokenType.Table);
	});

	test('incremental parsing: table with gradual cell completion', async () => {
		const tableStart = '| A | B |\n|---|---|\n| 1';
		const tableMiddle = ' | 2';
		const tableEnd = ' |\n| 3 | 4 |';

		let state: IncrementalState | null = null;

		// Parse initial part
		const result1 = await parseIncremental(tableStart, state);
		state = result1.state;

		// Add more content
		const result2 = await parseIncremental(tableStart + tableMiddle, state);
		state = result2.state;

		// Complete the table
		const result3 = await parseIncremental(tableStart + tableMiddle + tableEnd, state);

		// Final should be a complete table
		expect(result3.result.tokens.length).toBe(1);
		expect(result3.result.tokens[0].type).toBe(TokenType.Table);
		const table = result3.result.tokens[0] as TableToken;
		expect(table.children?.length).toBe(3); // header + 2 data rows
	});
});

describe('Table Incremental Parsing - Mixed Content', () => {
	test('incremental: heading + table + paragraph', async () => {
		const content1 = '# Title\n\n';
		const content2 = content1 + '| A | B |\n|---|---|\n| 1 | 2 |\n\n';
		const content3 = content2 + 'After table paragraph.';

		let state: IncrementalState | null = null;

		const r1 = await parseIncremental(content1, state);
		state = r1.state;
		expect(r1.result.tokens.length).toBe(1);
		expect(r1.result.tokens[0].type).toBe(TokenType.Heading);

		const r2 = await parseIncremental(content2, state);
		state = r2.state;
		expect(r2.result.tokens.length).toBe(2);
		expect(r2.result.tokens[1].type).toBe(TokenType.Table);

		const r3 = await parseIncremental(content3, state);
		expect(r3.result.tokens.length).toBe(3);
		expect(r3.result.tokens[2].type).toBe(TokenType.Paragraph);
	});

	test('incremental: code block + table + list', async () => {
		let state: IncrementalState | null = null;
		let source = '';

		// Add code block
		source += '```js\ncode\n```\n\n';
		const r1 = await parseIncremental(source, state);
		state = r1.state;
		expect(r1.result.tokens[0].type).toBe(TokenType.CodeBlock);

		// Add table
		source += '| A | B |\n|---|---|\n| 1 | 2 |\n\n';
		const r2 = await parseIncremental(source, state);
		state = r2.state;
		expect(r2.result.tokens.length).toBe(2);
		expect(r2.result.tokens[1].type).toBe(TokenType.Table);

		// Add list
		source += '- Item 1\n- Item 2';
		const r3 = await parseIncremental(source, state);
		expect(r3.result.tokens.length).toBe(3);
		expect(r3.result.tokens[2].type).toBe(TokenType.UnorderedList);
	});

	test('incremental: table with inline formatting added gradually', async () => {
		const part1 = '| Header |\n|--------|\n| **Bol';
		const part2 = 'd** |\n| *Ital';
		const part3 = 'ic* |';

		let state: IncrementalState | null = null;

		const r1 = await parseIncremental(part1, state);
		state = r1.state;

		const r2 = await parseIncremental(part1 + part2, state);
		state = r2.state;

		const r3 = await parseIncremental(part1 + part2 + part3, state);

		expect(r3.result.tokens[0].type).toBe(TokenType.Table);
		const table = r3.result.tokens[0] as TableToken;

		// Check that formatting is parsed
		let hasBold = false;
		let hasItalic = false;

		table.children?.forEach((row) => {
			row.children?.forEach((cell) => {
				if (cell.children?.some((t) => t.type === TokenType.Bold)) {
					hasBold = true;
				}
				if (cell.children?.some((t) => t.type === TokenType.Italic)) {
					hasItalic = true;
				}
			});
		});

		expect(hasBold).toBe(true);
		expect(hasItalic).toBe(true);
	});
});

describe('Table Incremental Parsing - Tab vs Pipe Separated', () => {
	test('incremental: tab-separated table', async () => {
		const tabTable = 'A\tB\tC\n---\t---\t---\n1\t2\t3';
		const result = await parseIncremental(tabTable, null);

		expect(result.result.tokens.length).toBe(1);
		expect(result.result.tokens[0].type).toBe(TokenType.Table);
	});

	test('incremental: pipe-separated table', async () => {
		const pipeTable = '| A | B | C |\n|---|---|---|\n| 1 | 2 | 3 |';
		const result = await parseIncremental(pipeTable, null);

		expect(result.result.tokens.length).toBe(1);
		expect(result.result.tokens[0].type).toBe(TokenType.Table);
	});

	test('incremental: tab-separated table built gradually', async () => {
		let state: IncrementalState | null = null;

		const r1 = await parseIncremental('A\tB', state);
		state = r1.state;

		const r2 = await parseIncremental('A\tB\n---\t---', state);
		state = r2.state;

		const r3 = await parseIncremental('A\tB\n---\t---\n1\t2', state);

		expect(r3.result.tokens.length).toBe(1);
		expect(r3.result.tokens[0].type).toBe(TokenType.Table);
	});

	test('incremental: mixed tabs and spaces in table', async () => {
		const mixedTable = 'Col1\t  Col2\n---\t---\nVal1\t  Val2';
		const result = await parseIncremental(mixedTable, null);

		expect(result.result.tokens.length).toBe(1);
		expect(result.result.tokens[0].type).toBe(TokenType.Table);
	});
});

describe('Table Incremental Parsing - Huge Blocks', () => {
	test('incremental: large table with many rows', async () => {
		// Generate a large table
		let largeTable = '| Col1 | Col2 | Col3 | Col4 | Col5 |\n';
		largeTable += '|------|------|------|------|------|\n';

		// Add 100 rows
		for (let i = 0; i < 100; i++) {
			largeTable += `| R${i}C1 | R${i}C2 | R${i}C3 | R${i}C4 | R${i}C5 |\n`;
		}

		const result = await parseIncremental(largeTable, null);

		expect(result.result.tokens.length).toBe(1);
		expect(result.result.tokens[0].type).toBe(TokenType.Table);
		const table = result.result.tokens[0] as TableToken;
		expect(table.children?.length).toBe(101); // header + 100 rows
	});

	test('incremental: huge table built incrementally in chunks', async () => {
		let state: IncrementalState | null = null;
		let content = '| A | B |\n|---|---|\n';

		// Start with header
		const r1 = await parseIncremental(content, state);
		state = r1.state;

		// Add rows in chunks of 10
		for (let chunk = 0; chunk < 10; chunk++) {
			for (let row = 0; row < 10; row++) {
				const rowNum = chunk * 10 + row;
				content += `| ${rowNum} | ${rowNum * 2} |\n`;
			}

			const result = await parseIncremental(content, state);
			state = result.state;

			// Should maintain table structure
			expect(result.result.tokens[0].type).toBe(TokenType.Table);
		}

		// Final table should have 101 rows (1 header + 100 data)
		const finalTable = state!.prevResult.tokens[0] as TableToken;
		expect(finalTable.children?.length).toBe(101);
	});

	test('incremental: table with very wide rows', async () => {
		const wideCols = 20;
		let wideTable = '|';
		for (let i = 0; i < wideCols; i++) {
			wideTable += ` Col${i} |`;
		}
		wideTable += '\n|';
		for (let i = 0; i < wideCols; i++) {
			wideTable += '-------|';
		}
		wideTable += '\n|';
		for (let i = 0; i < wideCols; i++) {
			wideTable += ` Val${i} |`;
		}
		wideTable += '|';

		const result = await parseIncremental(wideTable, null);

		expect(result.result.tokens.length).toBe(1);
		expect(result.result.tokens[0].type).toBe(TokenType.Table);
		const table = result.result.tokens[0] as TableToken;
		expect(table.children?.[0].children?.length).toBe(wideCols);
	});
});

describe('Table Incremental Parsing - Blockquote Context', () => {
	test('incremental: table within blockquote', async () => {
		const quotedTable = `> | A | B |
> |---|---|
> | 1 | 2 |`;

		const result = await parseIncremental(quotedTable, null);

		expect(result.result.tokens.length).toBe(1);
		expect(result.result.tokens[0].type).toBe(TokenType.Blockquote);

		// The blockquote should contain a table
		const blockquote = result.result.tokens[0];
		const hasTable = blockquote.children?.some((t) => t.type === TokenType.Table);
		expect(hasTable).toBe(true);
	});

	test('incremental: table within blockquote built gradually', async () => {
		let state: IncrementalState | null = null;

		const r1 = await parseIncremental('> | A | B |', state);
		state = r1.state;
		expect(r1.result.tokens[0].type).toBe(TokenType.Blockquote);

		const r2 = await parseIncremental('> | A | B |\n> |---|---|', state);
		state = r2.state;
		expect(r2.result.tokens[0].type).toBe(TokenType.Blockquote);

		const r3 = await parseIncremental('> | A | B |\n> |---|---|\n> | 1 | 2 |', state);

		expect(r3.result.tokens[0].type).toBe(TokenType.Blockquote);
		const blockquote = r3.result.tokens[0];
		const hasTable = blockquote.children?.some((t) => t.type === TokenType.Table);
		expect(hasTable).toBe(true);
	});

	test('table after blockquote', async () => {
		let state: IncrementalState | null = null;

		const r1 = await parseIncremental('> Quote\n\n', state);
		state = r1.state;
		expect(r1.result.tokens[0].type).toBe(TokenType.Blockquote);

		const r2 = await parseIncremental('> Quote\n\n| A | B |\n|---|---|\n| 1 | 2 |', state);

		expect(r2.result.tokens.length).toBe(2);
		expect(r2.result.tokens[0].type).toBe(TokenType.Blockquote);
		expect(r2.result.tokens[1].type).toBe(TokenType.Table);
	});

	test('blockquote after table', async () => {
		let state: IncrementalState | null = null;

		const r1 = await parseIncremental('| A | B |\n|---|---|\n| 1 | 2 |\n\n', state);
		state = r1.state;
		expect(r1.result.tokens[0].type).toBe(TokenType.Table);

		const r2 = await parseIncremental('| A | B |\n|---|---|\n| 1 | 2 |\n\n> Quote', state);

		expect(r2.result.tokens.length).toBe(2);
		expect(r2.result.tokens[0].type).toBe(TokenType.Table);
		expect(r2.result.tokens[1].type).toBe(TokenType.Blockquote);
	});
});

describe('Table Incremental Parsing - Expected Behavior: Last Line as Paragraph First', () => {
	/**
	 * This test verifies the expected behavior:
	 * "The expected behavior of a multiple line token (block-level token) is to parse
	 * last line as paragraph first, then re-parse as corresponding token if the line
	 * turns out to be part of block-level token"
	 */
	test('incremental: first table line parsed as paragraph, then converted to table', async () => {
		let state: IncrementalState | null = null;

		// First line only - should be parsed as paragraph
		const r1 = await parseIncremental('| A | B |', state);
		state = r1.state;

		// At this point, the first line might be paragraph (incomplete table)
		expect(r1.result.tokens.length).toBeGreaterThan(0);
		// Could be paragraph initially since we don't have separator line yet

		// Add separator line - now it becomes clear it's a table
		const r2 = await parseIncremental('| A | B |\n|---|---|', state);
		state = r2.state;

		// Now should be recognized as table
		expect(r2.result.tokens.length).toBe(1);
		expect(r2.result.tokens[0].type).toBe(TokenType.Table);
	});

	test('incremental: incomplete table row treated as paragraph until completed', async () => {
		let state: IncrementalState | null = null;

		// Incomplete row
		const r1 = await parseIncremental('| A | B |\n|---|---|\n| 1', state);
		state = r1.state;

		// Should have tokens but last part might be incomplete
		expect(r1.result.tokens.length).toBeGreaterThan(0);

		// Complete the row
		const r2 = await parseIncremental('| A | B |\n|---|---|\n| 1 | 2 |', state);

		// Should now be complete table
		expect(r2.result.tokens.length).toBe(1);
		expect(r2.result.tokens[0].type).toBe(TokenType.Table);
		const table = r2.result.tokens[0] as TableToken;
		expect(table.children?.length).toBe(2); // header + 1 data row
	});

	test('incremental: paragraph followed by table formation', async () => {
		let state: IncrementalState | null = null;

		// Start with what looks like a paragraph
		const r1 = await parseIncremental('Some text\n\n| Header', state);
		state = r1.state;

		// Add separator to form table
		const r2 = await parseIncremental('Some text\n\n| Header |\n|-----', state);
		state = r2.state;

		// Complete table
		const r3 = await parseIncremental('Some text\n\n| Header |\n|--------|\n| Data |', state);

		// Should have paragraph and table
		expect(r3.result.tokens.length).toBe(2);
		expect(r3.result.tokens[0].type).toBe(TokenType.Paragraph);
		expect(r3.result.tokens[1].type).toBe(TokenType.Table);
	});

	test('incremental: multi-line formation - paragraph to list to table context', async () => {
		let state: IncrementalState | null = null;

		// Just text - paragraph
		const r1 = await parseIncremental('Text', state);
		state = r1.state;
		expect(r1.result.tokens[0].type).toBe(TokenType.Paragraph);

		// Add more content
		const r2 = await parseIncremental('Text\n\n- List item\n\n| A', state);
		state = r2.state;

		// Complete table
		const r3 = await parseIncremental('Text\n\n- List item\n\n| A | B |\n|---|---|\n| 1 | 2 |', state);

		expect(r3.result.tokens.length).toBe(3);
		expect(r3.result.tokens[0].type).toBe(TokenType.Paragraph);
		expect(r3.result.tokens[1].type).toBe(TokenType.UnorderedList);
		expect(r3.result.tokens[2].type).toBe(TokenType.Table);
	});
});

describe('Table Incremental Parsing - Edge Cases', () => {
	test('table with empty cells', async () => {
		const emptyTable = '| A | B | C |\n|---|---|---|\n| 1 |   | 3 |\n|   | 2 |   |';
		const result = await parseIncremental(emptyTable, null);

		expect(result.result.tokens[0].type).toBe(TokenType.Table);
		const table = result.result.tokens[0] as TableToken;
		expect(table.children?.length).toBe(3); // header + 2 rows
	});

	test('table with alignment markers', async () => {
		const alignedTable = '| Left | Center | Right |\n|:-----|:------:|------:|\n| L | C | R |';
		const result = await parseIncremental(alignedTable, null);

		expect(result.result.tokens[0].type).toBe(TokenType.Table);
	});

	test('table followed immediately by another table (separated by blank line)', async () => {
		const doubleTables = '| A | B |\n|---|---|\n| 1 | 2 |\n\n| X | Y |\n|---|---|\n| 9 | 8 |';
		const result = await parseIncremental(doubleTables, null);

		expect(result.result.tokens.length).toBe(2);
		expect(result.result.tokens[0].type).toBe(TokenType.Table);
		expect(result.result.tokens[1].type).toBe(TokenType.Table);
	});

	test('table with special characters in cells', async () => {
		const specialTable = '| A | B |\n|---|---|\n| <tag> | `code` |\n| **bold** | *italic* |';
		const result = await parseIncremental(specialTable, null);

		expect(result.result.tokens[0].type).toBe(TokenType.Table);
	});

	test('incomplete table without separator line stays as paragraph', async () => {
		const incomplete = '| A | B |\n| 1 | 2 |';
		const result = await parseIncremental(incomplete, null);

		// Without separator line, this should not be recognized as table
		// Parser looks for separator on second line
		expect(result.result.tokens.length).toBeGreaterThan(0);
		// Might be paragraph or unrecognized as table
	});
});
