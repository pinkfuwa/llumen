import { describe, it, expect } from 'vitest';
import { readFileSync, existsSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import { parseSync } from '../../parser/block';
import { parseIncremental, type IncrementalState } from '../../incremental';
import { AstNodeType, type AstNode } from '../../parser/types';

const __dirname = dirname(fileURLToPath(import.meta.url));
const fixturesPath = resolve(__dirname, 'codegen.json');

interface Fixture {
	name: string;
	markdown: string;
	expectedTypes: string[];
	chunks: string[][];
}

function loadFixtures(): Fixture[] {
	if (!existsSync(fixturesPath)) {
		return [];
	}
	const raw = readFileSync(fixturesPath, 'utf-8');
	return JSON.parse(raw) as Fixture[];
}

const fixtures = loadFixtures();

function serializeTree(nodes: AstNode[]): unknown[] {
	return nodes.map((n) => {
		const obj: Record<string, unknown> = { type: n.type };
		if (n.children) {
			obj.children = serializeTree(n.children);
		}
		if ('level' in n) obj.level = (n as { level: number }).level;
		if ('language' in n) obj.language = (n as { language?: string }).language;
		if ('content' in n) obj.content = (n as { content: string }).content;
		if ('closed' in n) obj.closed = (n as { closed: boolean }).closed;
		if ('url' in n) obj.url = (n as { url: string }).url;
		if ('alt' in n) obj.alt = (n as { alt: string }).alt;
		if ('isHeader' in n) obj.isHeader = (n as { isHeader: boolean }).isHeader;
		if ('align' in n) obj.align = (n as { align?: string }).align;
		if ('startNumber' in n) obj.startNumber = (n as { startNumber?: number }).startNumber;
		return obj;
	});
}

describe.skipIf(fixtures.length === 0)('codegen: incremental parsing', () => {
	for (const fixture of fixtures) {
		for (let ci = 0; ci < fixture.chunks.length; ci++) {
			it(`${fixture.name} (chunk set ${ci}): incremental matches full parse`, () => {
				const chunks = fixture.chunks[ci];
				let accumulated = '';
				let state: Partial<IncrementalState> = {};

				for (const chunk of chunks) {
					accumulated += chunk;
					parseIncremental(accumulated, state);
				}

				expect(accumulated).toBe(fixture.markdown);

				const fullResult = parseSync(fixture.markdown);
				const incrementalResult = parseIncremental(fixture.markdown, {});

				expect(serializeTree(incrementalResult)).toEqual(serializeTree(fullResult.nodes));
			});
		}
	}
});

describe('incremental: basic scenarios', () => {
	it('full reparse when source does not start with previous', () => {
		const state: Partial<IncrementalState> = {};
		const result1 = parseIncremental('# Hello', state);
		const result2 = parseIncremental('## World', state);

		// Should have done a full reparse since "## World" doesn't start with "# Hello"
		expect(result2).toHaveLength(1);
		expect(result2[0].type).toBe(AstNodeType.Heading);
	});

	it('reuses state when only whitespace is appended', () => {
		const state: Partial<IncrementalState> = {};
		const result1 = parseIncremental('# Hello', state);
		const result2 = parseIncremental('# Hello\n\n', state);

		// Same content, just whitespace — should reuse previous result
		expect(result2).toHaveLength(result1.length);
	});

	it('handles streaming append of new paragraph', () => {
		const state: Partial<IncrementalState> = {};
		const src1 = '# Title\n\nFirst paragraph.\n\n';
		const result1 = parseIncremental(src1, state);

		const src2 = src1 + 'Second paragraph.';
		const result2 = parseIncremental(src2, state);

		const paragraphs = result2.filter((n) => n.type === AstNodeType.Paragraph);
		expect(paragraphs.length).toBeGreaterThanOrEqual(2);
	});
});
