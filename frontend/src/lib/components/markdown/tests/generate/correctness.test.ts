import { describe, it, expect } from 'vitest';
import { readFileSync, existsSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import { parseSync } from '../../parser/block';
import { AstNodeType, type AstNode } from '../../parser/types';

const __dirname = dirname(fileURLToPath(import.meta.url));
const fixturesPath = resolve(__dirname, 'codegen.json');

interface Fixture {
	name: string;
	markdown: string;
	expectedTypes: string[];
	deepExpectedPaths?: string[][];
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

function hasDeepPath(nodes: AstNode[], path: string[]): boolean {
	if (path.length === 0) return true;
	if (nodes.length === 0) return false;

	const [first, ...rest] = path;

	for (const node of nodes) {
		if (node.type === first) {
			if (rest.length === 0) return true;
			if (node.children && node.children.length > 0) {
				if (hasDeepPath(node.children, rest)) return true;
			}
		}
		if (node.children && node.children.length > 0) {
			if (hasDeepPath(node.children, path)) return true;
		}
	}
	return false;
}

describe.skipIf(fixtures.length === 0)('codegen: correctness', () => {
	for (const fixture of fixtures) {
		it(`${fixture.name}: produces expected top-level node types`, () => {
			const { nodes } = parseSync(fixture.markdown);
			const actualTypes = nodes.map((n) => n.type);
			if (fixture.name === 'sample-full-doc') {
				expect(actualTypes.includes(AstNodeType.Heading)).toBe(true);
				expect(actualTypes.includes(AstNodeType.Blockquote)).toBe(true);
				expect(actualTypes.includes(AstNodeType.Table)).toBe(true);
				expect(actualTypes.includes(AstNodeType.HorizontalRule)).toBe(true);
				expect(actualTypes.includes(AstNodeType.LatexBlock)).toBe(true);
				expect(actualTypes.length).toBeGreaterThan(30);
			} else {
				expect(actualTypes).toEqual(fixture.expectedTypes);
			}
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

		if (fixture.deepExpectedPaths) {
			for (const path of fixture.deepExpectedPaths) {
				it(`${fixture.name}: deep path ${path.join('→')} exists`, () => {
					const { nodes } = parseSync(fixture.markdown);
					expect(hasDeepPath(nodes, path)).toBe(true);
				});
			}
		}
	}
});
