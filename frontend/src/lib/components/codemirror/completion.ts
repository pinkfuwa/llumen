import type { CompletionContext } from '@codemirror/autocomplete';

// Constants for TOML completion
export const TOP_LEVEL_FIELDS = ['model_id', 'display_name'];
export const CAPABILITY_FIELDS = ['image', 'audio', 'ocr', 'tool'];
export const PARAMETER_FIELDS = ['temperature', 'repeat_penalty', 'top_k', 'top_p'];
export const TOML_TABLE_HEADERS = ['[capability]', '[parameter]'];

// Known boolean fields for autocomplete
export const BOOLEAN_FIELDS = ['image', 'audio', 'ocr', 'tool'];

// Model IDs are expected to be injected from outside (e.g., CodeMirror setup)
export let modelIds: string[] = [];
export let modelIdsFetched = false;

export async function fetchModelIds() {
	try {
		const resp = await fetch('https://openrouter.ai/api/v1/models');
		const { data: models } = await resp.json();
		modelIds = models.map((m: any) => m.id);
		modelIdsFetched = true;
	} catch (e) {
		modelIds = [];
		modelIdsFetched = true;
	}
}

/**
 * TOML completion function for CodeMirror.
 * Context-aware suggestions for ModelConfig TOML:
 * - Table headers ([capability], [parameter]) if missing
 * - Top-level fields (model_id, display_name)
 * - Capability/parameter fields inside respective tables
 * - Provider/model completion for model_id line
 */
export function tomlCompletion(context: CompletionContext) {
	const { state, pos } = context;
	const lineObj = state.doc.lineAt(pos);
	const lineText = lineObj.text;
	const before = lineText.slice(0, pos - lineObj.from);

	// Gather all table headers in the document
	const tableHeaders = new Set<string>();
	for (let i = 1; i <= state.doc.lines; i++) {
		const txt = state.doc.line(i).text.trim();
		const match = txt.match(/^\[(\w+)\]$/);
		if (match) {
			tableHeaders.add(`[${match[1]}]`);
		}
	}

	// Find current table context by scanning up from cursor
	let currentTable: string | null = null;
	for (let i = lineObj.number - 1; i >= 1; i--) {
		const txt = state.doc.line(i).text.trim();
		const match = txt.match(/^\[(\w+)\]$/);
		if (match) {
			currentTable = `[${match[1]}]`;
			break;
		}
	}

	// At start of line or after blank line, suggest only missing table headers
	if (/^\s*$/.test(before) || /^\s*\[/.test(before)) {
		const missingHeaders = TOML_TABLE_HEADERS.filter((h) => !tableHeaders.has(h));
		if (missingHeaders.length > 0) {
			return {
				from: lineObj.from,
				options: missingHeaders.map((h) => ({
					label: h,
					type: 'variable',
					info: 'TOML table header'
				})),
				validFor: /^\[.*$/
			};
		}
	}

	// If editing model_id line, show provider/model completions
	if (/^\s*model_id\s*=\s*["'`]?/.test(lineText)) {
		const eqIdx = lineText.indexOf('=');
		const afterEq = eqIdx >= 0 ? lineText.slice(eqIdx + 1).trim() : '';
		const quoteMatch = afterEq.match(/^(['"`])/);
		const quoteChar = quoteMatch ? quoteMatch[1] : '';
		const afterQuote = quoteChar ? afterEq.slice(1) : afterEq;
		const word = context.matchBefore(/[\w\/\-]*/);
		if (!word) return null;

		// If user typed provider/model, filter models. If only provider, show provider list.
		if (afterQuote.length === 0 || !/[a-zA-Z0-9]/.test(afterQuote)) {
			// Show only unique providers
			const providers = Array.from(new Set(modelIds.map((id) => id.split('/')[0])));
			return {
				from: word.from,
				options: providers.map((provider) => ({
					label: provider,
					type: 'variable',
					info: 'provider'
				}))
			};
		} else if (!afterQuote.includes('/')) {
			// Show providers matching prefix
			const prefix = afterQuote;
			const providers = Array.from(new Set(modelIds.map((id) => id.split('/')[0]))).filter(
				(provider) => provider.startsWith(prefix)
			);
			return {
				from: word.from,
				options: providers.map((provider) => ({
					label: provider,
					type: 'variable',
					info: 'provider'
				}))
			};
		} else {
			// Show models matching provider/model prefix
			const prefix = afterQuote;
			const filteredModels = modelIds.filter((id) => id.startsWith(prefix));
			return {
				from: word.from,
				options: filteredModels.map((id) => ({
					label: id,
					type: 'variable',
					info: 'provider/model'
				}))
			};
		}
	}

	// Top-level: only suggest model_id and display_name
	if (!currentTable) {
		const word = context.matchBefore(/\w+/);
		if (!word) return null;
		return {
			from: word.from,
			options: TOP_LEVEL_FIELDS.map((f) => ({
				label: f,
				type: 'variable'
			}))
		};
	}

	// Inside [capability]: only suggest capability fields
	if (currentTable === '[capability]') {
		// If completing a boolean field value, suggest true/false
		const fieldLineMatch = lineText.match(/^\s*(\w+)\s*=\s*$/);
		if (fieldLineMatch && BOOLEAN_FIELDS.includes(fieldLineMatch[1])) {
			return {
				from: pos,
				options: [
					{ label: 'true', type: 'keyword', info: 'boolean' },
					{ label: 'false', type: 'keyword', info: 'boolean' }
				]
			};
		}
		const word = context.matchBefore(/\w+/);
		if (!word) return null;
		return {
			from: word.from,
			options: CAPABILITY_FIELDS.map((f) => ({
				label: f,
				type: 'variable'
			}))
		};
	}

	// Inside [parameter]: only suggest parameter fields
	if (currentTable === '[parameter]') {
		const word = context.matchBefore(/\w+/);
		if (!word) return null;
		return {
			from: word.from,
			options: PARAMETER_FIELDS.map((f) => ({
				label: f,
				type: 'variable'
			}))
		};
	}

	return null;
}
