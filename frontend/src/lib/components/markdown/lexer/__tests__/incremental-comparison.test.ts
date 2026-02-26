import { describe, test, expect } from 'vitest';
import { parse, parseIncremental, type IncrementalState, type Token, TokenType } from '../index';

function seededRandom(seed: number): () => number {
	return () => {
		seed = (seed * 1103515245 + 12345) & 0x7fffffff;
		return seed / 0x7fffffff;
	};
}

function generateChunkPositions(sourceLength: number, seed: number, numChunks: number): number[] {
	const random = seededRandom(seed);
	const positions = [0];
	
	for (let i = 0; i < numChunks - 1; i++) {
		const chunkSize = Math.max(1, Math.floor(random() * (sourceLength / numChunks) * 2));
		const lastPos = positions[positions.length - 1];
		const newPos = Math.min(sourceLength, lastPos + chunkSize);
		if (newPos > lastPos && newPos < sourceLength) {
			positions.push(newPos);
		}
	}
	
	if (positions[positions.length - 1] < sourceLength) {
		positions.push(sourceLength);
	}
	
	return positions;
}

function getChunkPattern(source: string, seed: number, numChunks: number): string[] {
	const positions = generateChunkPositions(source.length, seed, numChunks);
	const chunks: string[] = [];
	
	for (let i = 1; i < positions.length; i++) {
		chunks.push(source.slice(positions[i - 1], positions[i]));
	}
	
	return chunks;
}

async function parseIncrementally(source: string, chunks: string[]): Promise<ReturnType<typeof parse>> {
	let state: IncrementalState | null = null;
	let accumulated = '';
	
	for (const chunk of chunks) {
		accumulated += chunk;
		const result = await parseIncremental(accumulated, state);
		state = result.state;
	}
	
	return state!.prevResult;
}

function verifyNoContentDuplication(source: string, tokens: Token[]): boolean {
	const allChars = new Map<number, string>();
	
	function collectChars(token: Token, depth = 0): void {
		if (depth > 10) return;
		
		if (token.start !== undefined && token.end !== undefined) {
			for (let i = token.start; i < token.end; i++) {
				const char = source[i];
				if (char) {
					if (!allChars.has(i)) {
						allChars.set(i, char);
					}
				}
			}
		}
		
		if (token.children) {
			for (const child of token.children) {
				collectChars(child, depth + 1);
			}
		}
	}
	
	for (const token of tokens) {
		collectChars(token);
	}
	
	return allChars.size === source.length;
}

describe('Incremental Parsing - Comparison Tests', () => {
	const testSources = [
		'Simple paragraph',
		'# Heading\n\nParagraph',
		'Hey there! 👋',
		"It's working",
		'🎉🎊✨',
		'🌍🌎🌏',
	];

	for (const source of testSources) {
		for (const seed of [42, 123, 456]) {
			for (const chunks of [3, 5]) {
				if (chunks > source.length) continue;
				
				test(`"${source.slice(0, 20)}..." seed=${seed} chunks=${chunks}`, async () => {
					const chunkPattern = getChunkPattern(source, seed, chunks);
					
					const result = await parseIncrementally(source, chunkPattern);
					
					const hasNoDuplication = verifyNoContentDuplication(source, result.tokens);
					
					expect(hasNoDuplication).toBe(true);
				});
			}
		}
	}
});
