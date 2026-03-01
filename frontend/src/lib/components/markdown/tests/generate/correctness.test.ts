import { describe, it, expect } from 'vitest';
import { readFileSync, existsSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import { parseSync } from '../../parser/block';
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

describe.skipIf(fixtures.length === 0)('codegen: correctness', () => {
	for (const fixture of fixtures) {
		it(`${fixture.name}: produces expected top-level node types`, () => {
			const { nodes } = parseSync(fixture.markdown);
			const actualTypes = nodes.map((n) => n.type);
			expect(actualTypes).toEqual(fixture.expectedTypes);
		});

		it(`${fixture.name}: parse does not throw`, () => {
			expect(() => parseSync(fixture.markdown)).not.toThrow();
		});

		it(`${fixture.name}: all nodes have valid start/end positions`, () => {
			const { nodes } = parseSync(fixture.markdown);
			for (const node of nodes) {
				expect(node.start).toBeGreaterThanOrEqual(0);
				expect(node.end).toBeGreaterThanOrEqual(node.start);
			}
		});
	}
});
