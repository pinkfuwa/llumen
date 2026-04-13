import { describe, it, expect, beforeEach } from 'vitest';
import { EditorState } from '@codemirror/state';
import { autocompletion, type Completion } from '@codemirror/autocomplete';
import { tomlCompletion, setModelIds } from './completion';
import { writable } from 'svelte/store';

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
	return tomlCompletion(context as never);
}

function completionLabels(result: ReturnType<typeof getCompletions>): string[] {
	return result?.options.map((option: Completion) => option.label) ?? [];
}

describe('TOML Completion', () => {
	beforeEach(() => {
		setModelIds(
			writable([
				'anthropic/claude-3-5-sonnet-20241022',
				'anthropic/claude-3-opus-20240229',
				'openai/gpt-4-turbo-preview',
				'openai/gpt-3.5-turbo',
				'google/gemini-pro'
			])
		);
	});

	it('completes provider/model ids for model_id', () => {
		const result = getCompletions('model_id = "anth|"');
		expect(completionLabels(result)).toContain('anthropic');
		expect(completionLabels(result)).not.toContain('openai');
	});

	it('completes full model ids after provider slash', () => {
		const result = getCompletions('model_id = "anthropic/|"');
		expect(completionLabels(result)).toContain('anthropic/claude-3-5-sonnet-20241022');
		expect(completionLabels(result)).not.toContain('openai/gpt-4-turbo-preview');
	});

	it('completes provider/model ids for media_gen image_model', () => {
		const result = getCompletions('[media_gen]\nimage_model = "open|"');
		expect(completionLabels(result)).toContain('openai');
	});

	it('completes provider/model ids for media_gen video_model', () => {
		const result = getCompletions('[media_gen]\nvideo_model = "openai/|"');
		expect(completionLabels(result)).toContain('openai/gpt-4-turbo-preview');
	});

	it('completes booleans for capability fields', () => {
		const result = getCompletions('[capability]\nimage = t|');
		expect(completionLabels(result)).toEqual(expect.arrayContaining(['true']));
		expect(completionLabels(result)).not.toContain('false');
	});

	it('completes known literal values for ocr', () => {
		const result = getCompletions('[capability]\nocr = "mis|"');
		expect(completionLabels(result)).toContain('mistral');
	});

	it('completes reasoning literals including booleans', () => {
		const result = getCompletions('[capability]\nreasoning = "|"');
		expect(completionLabels(result)).toContain('high');
		expect(completionLabels(result)).toContain('true');
		expect(completionLabels(result)).toContain('false');
	});

	it('supports media_gen table header and fields', () => {
		const result = getCompletions('[media|');
		expect(completionLabels(result)).toContain('[media_gen]');

		const fieldResult = getCompletions('[media_gen]\n|');
		expect(completionLabels(fieldResult)).toEqual(expect.arrayContaining(['image_model', 'video_model']));
	});

	it('keeps table header suggestions complete', () => {
		const result = getCompletions('|');
		expect(completionLabels(result)).toEqual(expect.arrayContaining(['[capability]', '[parameter]', '[media_gen]']));
	});

	it('handles empty completion result safely', () => {
		expect(getCompletions('display_name = "Test"\n|')).toBeDefined();
	});
});
