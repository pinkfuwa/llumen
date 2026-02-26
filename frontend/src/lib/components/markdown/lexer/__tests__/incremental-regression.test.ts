import { describe, test, expect } from 'vitest';
import { parse, parseIncremental, type IncrementalState, type Token, TokenType } from '../index';

async function typeCharByChar(source: string): Promise<ReturnType<typeof parse>> {
	let state: IncrementalState | null = null;
	let accumulated = '';
	
	for (let i = 0; i < source.length; i++) {
		accumulated += source[i];
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
				if (allChars.has(i)) {
					if (allChars.get(i) !== char) {
						console.log(`DUPLICATE at position ${i}: existing '${allChars.get(i)}', new '${char}'`);
					}
				}
				allChars.set(i, char);
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

describe('Incremental Parsing - Key Regression Tests', () => {
	const keyCases = [
		{ source: "Hey there! 👋 How's your day going?", desc: 'emoji bug' },
		{ source: '🎉🎊✨ celebration', desc: 'multiple emojis' },
		{ source: 'Hello 👋 World 🌍', desc: 'emoji between words' },
		{ source: 'Does `D(p||q)=D(q||p)`?', desc: 'double pipe in code' },
		{ source: '`code` with | pipe', desc: 'pipe after code' },
		{ source: '你好世界 🌍', desc: 'chinese + emoji' },
		{ source: '日本語テスト 🔥', desc: 'japanese + emoji' },
		{ source: 'line1\nline2\nline3', desc: 'LLM single newlines' },
		{ source: 'Simple paragraph', desc: 'simple text' },
		{ source: '# Heading\n\nParagraph', desc: 'heading + para' },
	];

	for (const { source, desc } of keyCases) {
		test(`${desc}: no content duplication`, async () => {
			const result = await typeCharByChar(source);
			
			const hasNoDuplication = verifyNoContentDuplication(source, result.tokens);
			
			expect(hasNoDuplication).toBe(true);
		});
	}
});
