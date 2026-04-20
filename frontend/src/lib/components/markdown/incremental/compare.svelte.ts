import type { AstNode } from '../parser';

function getLabel(node: AstNode): string {
	return `${node.start}:${node.end}:${node.type}`;
}

function patchASTNode(node: AstNode, source: AstNode) {
	if (getLabel(node) === getLabel(source)) return;

	patchASTNodes(node, source);

	// copy rest of attr

	for (const key in source) {
		if (key !== 'children') {
			Object.assign(node, { [key]: source[key as keyof AstNode] });
		}
	}
}

export function patchASTNodes(node: { children?: AstNode[] }, source: { children?: AstNode[] }) {
	if (node.children == undefined) node.children = [];

	let shorter = Math.min(node.children?.length ?? 0, source.children?.length ?? 0);

	for (let i = 0; i < shorter; i++) {
		patchASTNode(node.children[i], source.children![i]);
	}

	if (source.children?.length == shorter) {
		node.children.splice(shorter);
	} else if (source.children) {
		const appended = source.children!.slice(shorter).map((child) => child);
		node.children.push(...appended);
	}
}
