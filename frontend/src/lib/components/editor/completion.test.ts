import { describe, it, expect, beforeEach } from 'vitest';
import { tomlCompletion, setModelIds, type CompletionOption } from './completion';
import { writable } from 'svelte/store';

function getCompletions(text: string) {
	const pos = text.indexOf('|');
	if (pos === -1) throw new Error('Test string must contain | for cursor position');
	const cleanText = text.replace('|', '');
	return tomlCompletion({ text: cleanText, pos });
}

function completionLabels(result: ReturnType<typeof getCompletions>): string[] {
	return result?.options.map((option: CompletionOption) => option.label) ?? [];
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

	it('completes provider/model ids for task_model_id', () => {
		const result = getCompletions('task_model_id = "anth|"');
		expect(completionLabels(result)).toContain('anthropic');
	});

	it('completes full model ids after provider slash for task_model_id', () => {
		const result = getCompletions('task_model_id = "anthropic/|"');
		expect(completionLabels(result)).toContain('anthropic/claude-3-5-sonnet-20241022');
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
		expect(completionLabels(fieldResult)).toEqual(
			expect.arrayContaining(['image_model', 'video_model'])
		);
	});

	it('keeps table header suggestions complete', () => {
		const result = getCompletions('|');
		expect(completionLabels(result)).toEqual(
			expect.arrayContaining(['[capability]', '[parameter]', '[media_gen]'])
		);
	});

	it('handles empty completion result safely', () => {
		expect(getCompletions('display_name = "Test"\n|')).toBeDefined();
	});
});
