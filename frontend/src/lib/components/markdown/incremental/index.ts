import type { AstNode, ParseResult } from '../parser/types';
import { parseSync } from '../parser/block';
import { detectBoundaryForIncrementalParse, mergePartialParseResult } from './region';
export { patchASTNodes } from './compare.svelte';

export interface IncrementalState {
	prevSource: string;
	prevResult: ParseResult;
	newContentStart: number;
}

export interface IncrementalParseResult {
	result: ParseResult;
	state: IncrementalState;
}

export function parseIncremental(source: string, state: Partial<IncrementalState>): AstNode[] {
	if (state == null) {
		state = {};
	}

	if (state.prevResult == undefined || !source.startsWith(state.prevSource || '')) {
		const result = parseSync(source);
		state.prevSource = source;
		state.prevResult = result;
		state.newContentStart = source.length;
		return result.nodes;
	}

	const newContent = source.slice(state.prevSource!.length);
	if (newContent.length === 0) {
		return state.prevResult.nodes;
	}

	const boundary = detectBoundaryForIncrementalParse({
		source,
		prevSourceLength: state.prevSource!.length,
		prevResult: state.prevResult
	});

	if (boundary.shouldReparseFully) {
		const result = parseSync(source);
		state.prevSource = source;
		state.prevResult = result;
		state.newContentStart = source.length;
		return result.nodes;
	}

	const partialResult = parseSync(source.slice(boundary.stableBoundary));
	const result: ParseResult = mergePartialParseResult(
		state.prevResult,
		partialResult,
		boundary.stableBoundary
	);

	state.prevSource = source;
	state.prevResult = result;
	state.newContentStart = source.length;

	return result.nodes;
}
