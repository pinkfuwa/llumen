import type { CompletionContext, CompletionResult } from '@codemirror/autocomplete';
import { get, writable, type Readable } from 'svelte/store';

// Constants for TOML completion
export const TOP_LEVEL_FIELDS = ['model_id', 'display_name'];
export const CAPABILITY_FIELDS = ['image', 'audio', 'ocr', 'tool', 'reasoning'];
export const PARAMETER_FIELDS = ['temperature', 'repeat_penalty', 'top_k', 'top_p'];
export const TOML_TABLE_HEADERS = ['[capability]', '[parameter]'];

// Known boolean fields for autocomplete
export const BOOLEAN_FIELDS = ['image', 'audio', 'ocr', 'tool'];

// Model IDs getter - will be set by the Svelte component
let modelIds: Readable<string[]> = writable([]);

export function setModelIds(models: Readable<string[]>) {
	modelIds = models;
}

interface ParsedLine {
	// The full line text
	text: string;
	// Position of cursor relative to line start
	cursorOffset: number;
	// Text before cursor on this line
	beforeCursor: string;
	// Text after cursor on this line
	afterCursor: string;
	// Detected field name if any (e.g., "model_id", "image")
	fieldName: string | null;
	// Whether we're in a value position (after =)
	inValue: boolean;
	// Quote character if inside quotes (" or ' or `)
	quoteChar: string | null;
	// Text inside quotes before cursor (if in quoted value)
	valueBeforeCursor: string;
	// Text inside quotes after cursor (if in quoted value)
	valueAfterCursor: string;
}

/**
 * Parse the current line to understand context
 */
function parseLine(lineText: string, cursorOffset: number): ParsedLine {
	const beforeCursor = lineText.slice(0, cursorOffset);
	const afterCursor = lineText.slice(cursorOffset);

	// Check if we're on a field = value line
	const fieldMatch = beforeCursor.match(/^\s*(\w+)\s*=\s*(.*)$/);

	if (fieldMatch) {
		const fieldName = fieldMatch[1];
		const afterEquals = fieldMatch[2];

		// Check if we're inside quotes
		const quoteMatch = afterEquals.match(/^(['"`])/);
		if (quoteMatch) {
			const quoteChar = quoteMatch[1];
			const insideQuote = afterEquals.slice(1);

			// Find if there's a closing quote after cursor
			const closingQuoteInAfter = afterCursor.indexOf(quoteChar);
			const valueAfterCursor =
				closingQuoteInAfter >= 0 ? afterCursor.slice(0, closingQuoteInAfter) : afterCursor;

			return {
				text: lineText,
				cursorOffset,
				beforeCursor,
				afterCursor,
				fieldName,
				inValue: true,
				quoteChar,
				valueBeforeCursor: insideQuote,
				valueAfterCursor
			};
		} else {
			// Unquoted value
			return {
				text: lineText,
				cursorOffset,
				beforeCursor,
				afterCursor,
				fieldName,
				inValue: true,
				quoteChar: null,
				valueBeforeCursor: afterEquals,
				valueAfterCursor: ''
			};
		}
	}

	return {
		text: lineText,
		cursorOffset,
		beforeCursor,
		afterCursor,
		fieldName: null,
		inValue: false,
		quoteChar: null,
		valueBeforeCursor: '',
		valueAfterCursor: ''
	};
}

/**
 * Find the current table context by scanning upward from current line
 */
function findCurrentTable(state: any, lineNumber: number): string | null {
	for (let i = lineNumber - 1; i >= 1; i--) {
		const line = state.doc.line(i);
		const match = line.text.trim().match(/^\[(\w+)\]$/);
		if (match) {
			return `[${match[1]}]`;
		}
	}
	return null;
}

/**
 * Get all existing table headers in the document
 */
function getExistingTables(state: any): Set<string> {
	const tables = new Set<string>();
	for (let i = 1; i <= state.doc.lines; i++) {
		const line = state.doc.line(i);
		const match = line.text.trim().match(/^\[(\w+)\]$/);
		if (match) {
			tables.add(`[${match[1]}]`);
		}
	}
	return tables;
}

/**
 * Complete model_id values (provider or provider/model)
 */
function completeModelId(parsed: ParsedLine, context: CompletionContext): CompletionResult | null {
	const models = get(modelIds);
	const currentValue = parsed.valueBeforeCursor + parsed.valueAfterCursor;
	const prefix = parsed.valueBeforeCursor;

	// Calculate the replacement range
	// We want to replace from the start of the value to the end of the value
	const line = context.state.doc.lineAt(context.pos);

	// Find where the quote starts (or where value starts if unquoted)
	let valueStart: number;
	if (parsed.quoteChar) {
		const quotePos = parsed.beforeCursor.lastIndexOf(parsed.quoteChar);
		valueStart = line.from + quotePos + 1;
	} else {
		const equalsPos = parsed.beforeCursor.lastIndexOf('=');
		const afterEquals = parsed.beforeCursor.slice(equalsPos + 1).trimStart();
		valueStart =
			line.from +
			equalsPos +
			1 +
			(parsed.beforeCursor.slice(equalsPos + 1).length - afterEquals.length);
	}

	// Find where the value ends
	let valueEnd: number;
	if (parsed.quoteChar) {
		const closingQuotePos = parsed.afterCursor.indexOf(parsed.quoteChar);
		if (closingQuotePos >= 0) {
			valueEnd = context.pos + closingQuotePos;
		} else {
			valueEnd = context.pos + parsed.afterCursor.length;
		}
	} else {
		valueEnd = context.pos + parsed.afterCursor.trimEnd().length;
	}

	// Determine what to show based on current value
	if (!prefix || !prefix.includes('/')) {
		// Show providers (with or without filtering)
		const providers = Array.from(new Set(models.map((id) => id.split('/')[0])));
		const filtered = prefix ? providers.filter((p) => p.startsWith(prefix)) : providers;

		return {
			from: valueStart,
			to: valueEnd,
			options: filtered.map((provider) => ({
				label: provider,
				type: 'variable',
				info: 'provider',
				apply: parsed.quoteChar ? provider : `"${provider}"`
			}))
		};
	} else {
		// Show full model IDs matching the prefix
		const fullValue = prefix + parsed.valueAfterCursor;
		const filtered = models.filter((id) => id.startsWith(prefix));

		return {
			from: valueStart,
			to: valueEnd,
			options: filtered.map((id) => ({
				label: id,
				type: 'variable',
				info: 'provider/model',
				apply: parsed.quoteChar ? id : `"${id}"`
			}))
		};
	}
}

/**
 * Complete boolean values (true/false)
 */
function completeBooleanValue(
	parsed: ParsedLine,
	context: CompletionContext
): CompletionResult | null {
	const line = context.state.doc.lineAt(context.pos);

	// Find where the value starts (after =)
	const equalsPos = parsed.beforeCursor.lastIndexOf('=');
	const afterEquals = parsed.beforeCursor.slice(equalsPos + 1).trimStart();
	const valueStart =
		line.from +
		equalsPos +
		1 +
		(parsed.beforeCursor.slice(equalsPos + 1).length - afterEquals.length);

	return {
		from: valueStart,
		to: context.pos + parsed.afterCursor.split(/\s/)[0].length,
		options: [
			{ label: 'true', type: 'keyword', info: 'boolean' },
			{ label: 'false', type: 'keyword', info: 'boolean' }
		],
		validFor: /^(true|false|t|f)?$/
	};
}

/**
 * Complete field names
 */
function completeFieldName(
	fields: string[],
	context: CompletionContext,
	beforeCursor: string
): CompletionResult | null {
	const word = context.matchBefore(/\w*/);
	if (!word) return null;

	return {
		from: word.from,
		to: word.to,
		options: fields.map((f) => ({
			label: f,
			type: 'variable'
		})),
		validFor: /^\w*$/
	};
}

/**
 * Check if current table section has any field definitions
 */
function hasFieldsInCurrentTable(
	state: any,
	lineNumber: number,
	currentTable: string | null
): boolean {
	if (!currentTable) return false;

	// Scan from the line after the table header to current line
	let foundTableHeader = false;
	for (let i = 1; i < lineNumber; i++) {
		const line = state.doc.line(i);
		const text = line.text.trim();

		// Check if this is the current table header
		if (text === currentTable) {
			foundTableHeader = true;
			continue;
		}

		// If we found another table header, stop
		if (foundTableHeader && text.match(/^\[\w+\]$/)) {
			break;
		}

		// If we found the current table and there's a field definition
		if (foundTableHeader && text.match(/^\w+\s*=/)) {
			return true;
		}
	}

	return false;
}

/**
 * Complete table headers
 */
function completeTableHeader(
	context: CompletionContext,
	existingTables: Set<string>,
	currentTable: string | null,
	state: any,
	lineNumber: number
): CompletionResult | null {
	const line = context.state.doc.lineAt(context.pos);
	const beforeCursor = line.text.slice(0, context.pos - line.from);

	// Check if we're typing a table header
	const tableMatch = beforeCursor.match(/^(\s*)\[(\w*)$/);
	if (tableMatch) {
		const indent = tableMatch[1];
		const partial = tableMatch[2];
		const missingHeaders = TOML_TABLE_HEADERS.filter((h) => !existingTables.has(h));

		return {
			from: line.from + indent.length,
			to: context.pos + line.text.slice(context.pos - line.from).indexOf(']') + 1 || context.pos,
			options: missingHeaders.map((h) => ({
				label: h,
				type: 'variable',
				info: 'TOML table header'
			})),
			validFor: /^\[\w*\]?$/
		};
	}

	// At start of line or blank line - suggest missing table headers
	// Only if:
	// 1. We're not inside a table section OR
	// 2. We're inside a table that already has fields (user might want to start new section)
	if (/^\s*$/.test(beforeCursor)) {
		const hasFields = hasFieldsInCurrentTable(state, lineNumber, currentTable);
		const shouldSuggestTables = !currentTable || hasFields;

		if (shouldSuggestTables) {
			const missingHeaders = TOML_TABLE_HEADERS.filter((h) => !existingTables.has(h));
			if (missingHeaders.length > 0) {
				return {
					from: line.from,
					to: context.pos,
					options: missingHeaders.map((h) => ({
						label: h,
						type: 'variable',
						info: 'TOML table header'
					}))
				};
			}
		}
	}

	return null;
}

/**
 * Main TOML completion function for CodeMirror
 */
export function tomlCompletion(context: CompletionContext): CompletionResult | null {
	const { state, pos } = context;
	const line = state.doc.lineAt(pos);
	const cursorOffset = pos - line.from;
	const parsed = parseLine(line.text, cursorOffset);

	// Get document context
	const currentTable = findCurrentTable(state, line.number);
	const existingTables = getExistingTables(state);

	// If we're in a value position
	if (parsed.inValue && parsed.fieldName) {
		// model_id completion
		if (parsed.fieldName === 'model_id') {
			return completeModelId(parsed, context);
		}

		// Boolean field completion
		if (BOOLEAN_FIELDS.includes(parsed.fieldName)) {
			return completeBooleanValue(parsed, context);
		}

		// Other values - no completion
		return null;
	}

	// Table header completion (passing currentTable and line info for context awareness)
	const tableResult = completeTableHeader(
		context,
		existingTables,
		currentTable,
		state,
		line.number
	);
	if (tableResult) return tableResult;

	// Field name completion based on context
	if (!currentTable) {
		// Top-level fields
		return completeFieldName(TOP_LEVEL_FIELDS, context, parsed.beforeCursor);
	} else if (currentTable === '[capability]') {
		// Capability fields
		return completeFieldName(CAPABILITY_FIELDS, context, parsed.beforeCursor);
	} else if (currentTable === '[parameter]') {
		// Parameter fields
		return completeFieldName(PARAMETER_FIELDS, context, parsed.beforeCursor);
	}

	return null;
}
