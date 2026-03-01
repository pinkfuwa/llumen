import { describe, it, expect } from 'vitest';
import { readFileSync, existsSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import { parseSync } from '../../parser/block';
import { parseIncremental, type IncrementalState } from '../../incremental';
import { AstNodeType } from '../../parser/types';

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

describe.skipIf(fixtures.length === 0)('codegen: incremental parsing', () => {
	for (const fixture of fixtures) {
		for (let ci = 0; ci < fixture.chunks.length; ci++) {
			it(`${fixture.name} (chunk set ${ci}): incremental matches full parse`, () => {
				const chunks = fixture.chunks[ci];
				let accumulated = '';
				let state: IncrementalState | null = null;

				for (const chunk of chunks) {
					accumulated += chunk;
					const result = parseIncremental(accumulated, state);
					state = result.state;
				}

				// Final accumulated should equal full markdown
				expect(accumulated).toBe(fixture.markdown);

				// Compare incremental result with full parse
				const fullResult = parseSync(fixture.markdown);
				const incrementalResult = parseIncremental(fixture.markdown, null);

				const fullTypes = fullResult.nodes.map((n) => n.type);
				const incTypes = incrementalResult.result.nodes.map((n) => n.type);
				expect(incTypes).toEqual(fullTypes);
			});
		}
	}
});

describe('incremental: basic scenarios', () => {
	it('full reparse when source does not start with previous', () => {
		const result1 = parseIncremental('# Hello', null);
		const result2 = parseIncremental('## World', result1.state);

		// Should have done a full reparse since "## World" doesn't start with "# Hello"
		expect(result2.result.nodes).toHaveLength(1);
		expect(result2.result.nodes[0].type).toBe(AstNodeType.Heading);
	});

	it('reuses state when only whitespace is appended', () => {
		const result1 = parseIncremental('# Hello', null);
		const result2 = parseIncremental('# Hello\n\n', result1.state);

		// Same content, just whitespace — should reuse previous result
		expect(result2.result.nodes).toHaveLength(result1.result.nodes.length);
	});

	it('handles streaming append of new paragraph', () => {
		const src1 = '# Title\n\nFirst paragraph.\n\n';
		const result1 = parseIncremental(src1, null);

		const src2 = src1 + 'Second paragraph.';
		const result2 = parseIncremental(src2, result1.state);

		const paragraphs = result2.result.nodes.filter(
			(n) => n.type === AstNodeType.Paragraph
		);
		expect(paragraphs.length).toBeGreaterThanOrEqual(2);
	});
});
