import { describe, it, expect, beforeEach } from 'vitest';
import { EditorState } from '@codemirror/state';
import { autocompletion } from '@codemirror/autocomplete';
import { tomlCompletion, setModelIds } from './completion';
import { writable } from 'svelte/store';

// Helper to create a state with cursor at | position
function createState(text: string): { state: EditorState; pos: number } {
	const pos = text.indexOf('|');
	if (pos === -1) throw new Error('Test string must contain | for cursor position');
	const cleanText = text.replace('|', '');
	const state = EditorState.create({
		doc: cleanText,
		extensions: [autocompletion({ override: [tomlCompletion] })]
	});
	return { state, pos };
}

// Helper to get completions at cursor
function getCompletions(text: string) {
	const { state, pos } = createState(text);
	const context = {
		state,
		pos,
		explicit: true,
		matchBefore: (regexp: RegExp) => {
			const line = state.doc.lineAt(pos);
			const textBefore = state.sliceDoc(line.from, pos);
			const match = textBefore.match(new RegExp(regexp.source + '$'));
			if (!match) return null;
			return {
				from: pos - match[0].length,
				to: pos,
				text: match[0]
			};
		}
	};
	return tomlCompletion(context as any);
}

describe('TOML Completion', () => {
	beforeEach(() => {
		const models = writable([
			'anthropic/claude-3-5-sonnet-20241022',
			'anthropic/claude-3-opus-20240229',
			'openai/gpt-4-turbo-preview',
			'openai/gpt-3.5-turbo',
			'google/gemini-pro'
		]);
		setModelIds(models);
	});

	describe('model_id completion - quoted strings', () => {
		it('should complete inside empty double quotes', () => {
			const result = getCompletions('model_id="|"');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'anthropic')).toBe(true);
		});

		it('should complete inside empty single quotes', () => {
			const result = getCompletions("model_id='|'");
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'anthropic')).toBe(true);
		});

		it('should complete with space after equals and inside quotes', () => {
			const result = getCompletions('model_id = "|"');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'anthropic')).toBe(true);
		});

		it('should complete partial provider inside quotes', () => {
			const result = getCompletions('model_id="anth|"');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'anthropic')).toBe(true);
		});

		it('should complete after provider and slash inside quotes', () => {
			const result = getCompletions('model_id="anthropic/|"');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'anthropic/claude-3-5-sonnet-20241022')).toBe(
				true
			);
		});

		it('should complete partial model inside quotes', () => {
			const result = getCompletions('model_id="anthropic/claude-3-5|"');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'anthropic/claude-3-5-sonnet-20241022')).toBe(
				true
			);
		});

		it('should complete with indentation', () => {
			const result = getCompletions('  model_id = "|"');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'anthropic')).toBe(true);
		});

		it('should complete at cursor between partial text and closing quote', () => {
			const result = getCompletions('model_id="open|"');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'openai')).toBe(true);
		});
	});

	describe('model_id completion - unquoted', () => {
		it('should complete without quotes', () => {
			const result = getCompletions('model_id=|');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'anthropic')).toBe(true);
		});

		it('should complete partial provider without quotes', () => {
			const result = getCompletions('model_id=anth|');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'anthropic')).toBe(true);
		});
	});

	describe('boolean field completion', () => {
		it('should complete boolean value after equals', () => {
			const result = getCompletions('[capability]\nimage = |');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'true')).toBe(true);
			expect(result?.options.some((o) => o.label === 'false')).toBe(true);
		});

		it('should complete boolean value with partial text', () => {
			const result = getCompletions('[capability]\nimage = t|');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'true')).toBe(true);
		});

		it('should complete boolean value without space', () => {
			const result = getCompletions('[capability]\nimage=|');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'true')).toBe(true);
		});
	});

	describe('field name completion', () => {
		it('should complete top-level field at start of line', () => {
			const result = getCompletions('|');
			expect(result).not.toBeNull();
			// In an empty document, both table headers and field names are valid
			// We currently prioritize table headers for structure-first approach
			const hasFields = result?.options.some(
				(o) => o.label === 'model_id' || o.label === 'display_name'
			);
			const hasTables = result?.options.some(
				(o) => o.label === '[capability]' || o.label === '[parameter]'
			);
			expect(hasFields || hasTables).toBe(true);
		});

		it('should complete partial top-level field', () => {
			const result = getCompletions('mod|');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'model_id')).toBe(true);
		});

		it('should complete field even when equals is present after cursor', () => {
			const result = getCompletions('mod| = "value"');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'model_id')).toBe(true);
		});

		it('should complete capability field', () => {
			const result = getCompletions('[capability]\n|');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'image')).toBe(true);
			expect(result?.options.some((o) => o.label === 'audio')).toBe(true);
		});

		it('should complete parameter field', () => {
			const result = getCompletions('[parameter]\n|');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'temperature')).toBe(true);
			expect(result?.options.some((o) => o.label === 'top_p')).toBe(true);
		});
	});

	describe('table header completion', () => {
		it('should suggest missing table headers at start of line', () => {
			const result = getCompletions('model_id = "test"\n|');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === '[capability]')).toBe(true);
			expect(result?.options.some((o) => o.label === '[parameter]')).toBe(true);
		});

		it('should suggest only missing table headers', () => {
			const result = getCompletions('[capability]\nimage = true\n|');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === '[parameter]')).toBe(true);
			expect(result?.options.some((o) => o.label === '[capability]')).toBe(false);
		});

		it('should complete partial table header', () => {
			const result = getCompletions('[cap|');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === '[capability]')).toBe(true);
		});

		it('should complete partial table header with closing bracket', () => {
			const result = getCompletions('[cap|]');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === '[capability]')).toBe(true);
		});
	});

	describe('context awareness', () => {
		it('should not suggest capability fields in parameter section', () => {
			const result = getCompletions('[parameter]\n|');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'image')).toBe(false);
			expect(result?.options.some((o) => o.label === 'temperature')).toBe(true);
		});

		it('should not suggest parameter fields in capability section', () => {
			const result = getCompletions('[capability]\n|');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'temperature')).toBe(false);
			expect(result?.options.some((o) => o.label === 'image')).toBe(true);
		});

		it('should handle multiple sections correctly', () => {
			const result = getCompletions('[capability]\nimage = true\n[parameter]\n|');
			expect(result).not.toBeNull();
			expect(result?.options.some((o) => o.label === 'temperature')).toBe(true);
			expect(result?.options.some((o) => o.label === 'image')).toBe(false);
		});
	});

	describe('edge cases', () => {
		it('should handle empty document', () => {
			const result = getCompletions('|');
			expect(result).not.toBeNull();
		});

		it('should handle cursor at end of document', () => {
			const result = getCompletions('model_id = "test"|');
			// Should not crash
			expect(result).toBeDefined();
		});

		it('should handle malformed input gracefully', () => {
			const result = getCompletions('model_id = "test\n[|');
			// Should not crash
			expect(result).toBeDefined();
		});
	});
});
