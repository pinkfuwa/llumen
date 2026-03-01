import type { AstNode, ParseResult, RegionBoundary } from '../parser/types';
import { parseSync } from '../parser/block';

const MIN_TRAILING_CHARS = 2;

export interface IncrementalState {
	prevSource: string;
	prevResult: ParseResult;
	newContentStart: number;
}

export interface IncrementalParseResult {
	result: ParseResult;
	state: IncrementalState;
}

export function parseIncremental(
	source: string,
	state: IncrementalState | null
): IncrementalParseResult {
	if (!state || !source.startsWith(state.prevSource)) {
		const result = parseSync(source);
		return {
			result,
			state: {
				prevSource: source,
				prevResult: result,
				newContentStart: source.length
			}
		};
	}

	const newContent = source.slice(state.prevSource.length);
	if (newContent.trim().length === 0) {
		return {
			result: state.prevResult,
			state: {
				prevSource: source,
				prevResult: state.prevResult,
				newContentStart: source.length
			}
		};
	}

	const stableBoundary = findLastStableBoundary(state.prevResult, state.prevSource.length);

	if (stableBoundary === 0) {
		const result = parseSync(source);
		return {
			result,
			state: {
				prevSource: source,
				prevResult: result,
				newContentStart: source.length
			}
		};
	}

	const stableNodes = state.prevResult.nodes.filter((node: AstNode) => node.end <= stableBoundary);

	const newContentToParse = source.slice(stableBoundary);
	const { nodes: newNodes, regions: newRegions } = parseSync(newContentToParse);

	const adjustedNewNodes = newNodes.map((node) => adjustNodePosition(node, stableBoundary));
	const adjustedNewRegions = newRegions.map((region) => ({
		...region,
		start: region.start + stableBoundary,
		end: region.end + stableBoundary
	}));

	const combinedNodes = [...stableNodes, ...adjustedNewNodes];
	const combinedRegions = [
		...state.prevResult.regions.filter((r: RegionBoundary) => r.end <= stableBoundary),
		...adjustedNewRegions
	];

	const result: ParseResult = {
		nodes: combinedNodes,
		regions: combinedRegions
	};

	return {
		result,
		state: {
			prevSource: source,
			prevResult: result,
			newContentStart: source.length
		}
	};
}

function findLastStableBoundary(result: ParseResult, sourceLength: number): number {
	if (result.nodes.length === 0) {
		return 0;
	}

	const threshold = sourceLength - MIN_TRAILING_CHARS;
	const minStableGap = MIN_TRAILING_CHARS * 3;

	if (result.regions.length > 0) {
		for (let i = result.regions.length - 1; i >= 0; i--) {
			const region = result.regions[i];
			const gapAfterRegion = sourceLength - region.end;
			if (region.end < threshold && gapAfterRegion >= minStableGap) {
				if (checkForBlankLineAfterRegion(result, region, sourceLength)) {
					return region.end;
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
				return node.end;
			}
		}
	}

	if (result.nodes.length > 1) {
		const secondLastNode = result.nodes[result.nodes.length - 2];
		const gapAfterSecondLast = sourceLength - secondLastNode.end;
		if (gapAfterSecondLast >= minStableGap) {
			return secondLastNode.end;
		}
	}

	return 0;
}

function checkForBlankLineAfterRegion(
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

function adjustNodePosition(node: AstNode, offset: number): AstNode {
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
