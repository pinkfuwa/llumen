import type { Token, ParseResult, RegionBoundary } from '../tokens';
import { parseBlocks } from './block-parser';

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

export async function parseIncremental(
	source: string,
	state: IncrementalState | null
): Promise<IncrementalParseResult> {
	if (!state || !source.startsWith(state.prevSource)) {
		const result = parse(source);
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
		const result = parse(source);
		return {
			result,
			state: {
				prevSource: source,
				prevResult: result,
				newContentStart: source.length
			}
		};
	}

	const stableTokens = state.prevResult.tokens.filter(
		(token: Token) => token.end <= stableBoundary
	);

	const newContentToParse = source.slice(stableBoundary);
	const { tokens: newTokens, regions: newRegions } = parseBlocks(newContentToParse, 0);

	const adjustedNewTokens = newTokens.map((token) => adjustTokenPosition(token, stableBoundary));

	const adjustedNewRegions = newRegions.map((region) => ({
		...region,
		start: region.start + stableBoundary,
		end: region.end + stableBoundary
	}));

	const combinedTokens = [...stableTokens, ...adjustedNewTokens];
	const combinedRegions = [
		...state.prevResult.regions.filter((r: RegionBoundary) => r.end <= stableBoundary),
		...adjustedNewRegions
	];

	const result: ParseResult = {
		tokens: combinedTokens,
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

export function parse(source: string): ParseResult {
	const { tokens, regions } = parseBlocks(source, 0);
	return { tokens, regions };
}

function findLastStableBoundary(result: ParseResult, sourceLength: number): number {
	if (result.tokens.length === 0) {
		return 0;
	}

	const threshold = sourceLength - MIN_TRAILING_CHARS;
	const minStableGap = MIN_TRAILING_CHARS * 3;

	if (result.regions.length > 0) {
		for (let i = result.regions.length - 1; i >= 0; i--) {
			const region = result.regions[i];
			const gapAfterRegion = sourceLength - region.end;
			if (region.end < threshold && gapAfterRegion >= minStableGap) {
				const hasBlankLineAfter = checkForBlankLineAfterRegion(result, region, sourceLength);
				if (hasBlankLineAfter) {
					return region.end;
				}
			}
		}
	}

	const lastToken = result.tokens[result.tokens.length - 1];
	if (lastToken.end >= threshold) {
		return 0;
	}

	const gapAfterLastToken = sourceLength - lastToken.end;
	if (gapAfterLastToken < minStableGap) {
		return 0;
	}

	for (let i = result.tokens.length - 2; i >= 0; i--) {
		const token = result.tokens[i];
		const nextToken = result.tokens[i + 1];

		if (token.end < threshold && nextToken && nextToken.start > token.end) {
			const gapAfterToken = nextToken.start - token.end;
			if (gapAfterToken >= minStableGap) {
				return token.end;
			}
		}
	}

	if (result.tokens.length > 1) {
		const secondLastToken = result.tokens[result.tokens.length - 2];
		const gapAfterSecondLast = sourceLength - secondLastToken.end;
		if (gapAfterSecondLast >= minStableGap) {
			return secondLastToken.end;
		}
	}

	return 0;
}

function checkForBlankLineAfterRegion(
	result: ParseResult,
	region: RegionBoundary,
	sourceLength: number
): boolean {
	const afterRegion = result.tokens.filter((t) => t.start >= region.end && t.end <= sourceLength);
	if (afterRegion.length === 0) {
		return false;
	}
	const firstTokenAfter = afterRegion[0];
	return firstTokenAfter.start - region.end >= MIN_TRAILING_CHARS * 2;
}

function adjustTokenPosition(token: Token, offset: number): Token {
	const adjusted = {
		...token,
		start: token.start + offset,
		end: token.end + offset
	};

	if (token.children) {
		adjusted.children = token.children.map((child: Token) => adjustTokenPosition(child, offset));
	}

	return adjusted;
}
