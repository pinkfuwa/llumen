import type { AstNode, ParseResult, RegionBoundary } from '../parser/types';
import { AstNodeType } from '../parser/types';

const MIN_TRAILING_CHARS = 2;
const TABLE_SEPARATOR_REGEX = /\|[\s\-:]+\|/;

export interface BoundaryDetectionInput {
	source: string;
	prevSourceLength: number;
	prevResult: ParseResult;
}

export interface BoundaryDetectionResult {
	shouldReparseFully: boolean;
	stableBoundary: number;
}

export function detectBoundaryForIncrementalParse({
	source,
	prevSourceLength,
	prevResult
}: BoundaryDetectionInput): BoundaryDetectionResult {
	if (
		TABLE_SEPARATOR_REGEX.test(source) ||
		hasTableInChangedRegion(prevResult, prevSourceLength, source.length)
	) {
		return {
			shouldReparseFully: true,
			stableBoundary: 0
		};
	}

	const stableBoundary = findLastStableBoundary(prevResult, prevSourceLength);
	if (stableBoundary === 0) {
		return {
			shouldReparseFully: true,
			stableBoundary: 0
		};
	}

	return {
		shouldReparseFully: false,
		stableBoundary
	};
}

export function hasTableInChangedRegion(
	prevResult: ParseResult,
	prevSourceLength: number,
	newSourceLength: number
): boolean {
	const changedStart = prevSourceLength;
	const changedEnd = newSourceLength;

	for (const node of prevResult.nodes) {
		if (node.type === AstNodeType.Table) {
			if (node.start < changedEnd && node.end > changedStart) {
				return true;
			}
		}
	}
	return false;
}

export function findLastStableBoundary(result: ParseResult, sourceLength: number): number {
	if (result.nodes.length === 0) {
		return 0;
	}

	const threshold = sourceLength - MIN_TRAILING_CHARS;
	const minStableGap = MIN_TRAILING_CHARS * 3;

	let candidateBoundary = 0;

	if (result.regions.length > 0) {
		for (let i = result.regions.length - 1; i >= 0; i--) {
			const region = result.regions[i];
			const gapAfterRegion = sourceLength - region.end;
			if (region.end < threshold && gapAfterRegion >= minStableGap) {
				if (checkForBlankLineAfterRegion(result, region, sourceLength)) {
					candidateBoundary = region.end;
					if (!boundaryCrossesTable(result.nodes, candidateBoundary)) {
						return candidateBoundary;
					}
				}
			}
		}
	}

	const lastNode = result.nodes[result.nodes.length - 1];
	if (lastNode.end >= threshold) {
		return 0;
	}

	const gapAfterLastNode = sourceLength - lastNode.end;
	if (gapAfterLastNode < minStableGap) {
		return 0;
	}

	for (let i = result.nodes.length - 2; i >= 0; i--) {
		const node = result.nodes[i];
		const nextNode = result.nodes[i + 1];

		if (node.end < threshold && nextNode && nextNode.start > node.end) {
			const gapAfterNode = nextNode.start - node.end;
			if (gapAfterNode >= minStableGap) {
				candidateBoundary = node.end;
				if (!boundaryCrossesTable(result.nodes, candidateBoundary)) {
					return candidateBoundary;
				}
			}
		}
	}

	if (result.nodes.length > 1) {
		const secondLastNode = result.nodes[result.nodes.length - 2];
		const gapAfterSecondLast = sourceLength - secondLastNode.end;
		if (gapAfterSecondLast >= minStableGap) {
			candidateBoundary = secondLastNode.end;
			if (!boundaryCrossesTable(result.nodes, candidateBoundary)) {
				return candidateBoundary;
			}
		}
	}

	return 0;
}

function boundaryCrossesTable(nodes: AstNode[], boundary: number): boolean {
	for (const node of nodes) {
		if (node.type === AstNodeType.Table) {
			if (node.start < boundary && node.end > boundary) {
				return true;
			}
		}
		if (node.children) {
			if (boundaryCrossesTable(node.children, boundary)) {
				return true;
			}
		}
	}
	return false;
}

export function checkForBlankLineAfterRegion(
	result: ParseResult,
	region: RegionBoundary,
	sourceLength: number
): boolean {
	const afterRegion = result.nodes.filter((n) => n.start >= region.end && n.end <= sourceLength);
	if (afterRegion.length === 0) {
		return false;
	}
	const firstNodeAfter = afterRegion[0];
	return firstNodeAfter.start - region.end >= MIN_TRAILING_CHARS * 2;
}

export function mergePartialParseResult(
	prevResult: ParseResult,
	partialResult: ParseResult,
	stableBoundary: number
): ParseResult {
	const stableNodes = prevResult.nodes.filter((node) => node.end <= stableBoundary);
	const adjustedNewNodes = partialResult.nodes.map((node) =>
		adjustNodePosition(node, stableBoundary)
	);
	const adjustedNewRegions = partialResult.regions.map((region) => ({
		...region,
		start: region.start + stableBoundary,
		end: region.end + stableBoundary
	}));

	return {
		nodes: [...stableNodes, ...adjustedNewNodes],
		regions: [
			...prevResult.regions.filter((region) => region.end <= stableBoundary),
			...adjustedNewRegions
		]
	};
}

export function adjustNodePosition(node: AstNode, offset: number): AstNode {
	const adjusted: AstNode = {
		...node,
		start: node.start + offset,
		end: node.end + offset
	};

	if (node.children) {
		adjusted.children = node.children.map((child: AstNode) => adjustNodePosition(child, offset));
	}

	return adjusted;
}
