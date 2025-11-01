import type { Tree } from '@lezer/common';

let parser: Promise<typeof import('./parser')> | null = null;

function initParser() {
	if (parser == null) {
		parser = import('./parser');
	}
	return parser;
}

export async function parseIncremental(
	prevTree: Tree,
	prevSource: string,
	source: string
): Promise<Tree> {
	return initParser().then((parser) => parser.parseIncremental(prevTree, prevSource, source));
}

export async function parse(source: string) {
	return initParser().then((parser) => parser.parse(source));
}

export async function walkTree(tree: Tree | null, source: string): Promise<any | null> {
	return initParser().then((parser) => parser.walkTree(tree, source));
}
