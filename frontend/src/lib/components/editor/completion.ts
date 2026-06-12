import { modelIds } from '$lib/api';

export const TOP_LEVEL_FIELDS = ['model_id', 'display_name', 'task_model_id'];
export const CAPABILITY_FIELDS = [
	'image',
	'audio',
	'video',
	'ocr',
	'tool',
	'json',
	'reasoning',
	'web'
];

export const WEB_OPTIONS = ['openrouter', 'native', 'builtIn', 'disabled'];
export const PARAMETER_FIELDS = ['temperature', 'repeat_penalty', 'top_k', 'top_p'];
export const MEDIA_GEN_FIELDS = ['image_model', 'video_model'];
export const TOML_TABLE_HEADERS = ['[capability]', '[parameter]', '[media_gen]'];

const BOOLEAN_FIELD_VALUES = ['true', 'false'];
const OCR_FIELD_VALUES = ['native', 'text', 'mistral', 'cloudflare', 'disabled'];
const REASONING_FIELD_VALUES = ['true', 'false', 'low', 'medium', 'high', 'none', 'auto'];
const MODEL_ID_FIELDS = new Set(['model_id', 'image_model', 'video_model', 'task_model_id']);
const BOOLEAN_FIELDS = new Set(['image', 'audio', 'video', 'tool', 'json']);
const STRING_VALUE_FIELDS = new Map([
	['ocr', OCR_FIELD_VALUES],
	['reasoning', REASONING_FIELD_VALUES],
	['web', WEB_OPTIONS]
]);

export interface CompletionOption {
	label: string;
	type: 'variable' | 'keyword';
	info?: string;
	apply?: string;
}

export interface CompletionResult {
	start: number;
	end: number;
	options: CompletionOption[];
}

export interface CompletionInput {
	text: string;
	pos: number;
}

interface ParsedLine {
	text: string;
	cursorOffset: number;
	beforeCursor: string;
	afterCursor: string;
	fieldName: string | null;
	inValue: boolean;
	quoteChar: string | null;
	valueBeforeCursor: string;
	valueAfterCursor: string;
}

function buildValueCompletionOptions(
	values: string[],
	quoted: boolean,
	info: string,
	wrapInQuotes: boolean
): CompletionOption[] {
	return values.map((value) => ({
		label: value,
		type: value === 'true' || value === 'false' ? 'keyword' : 'variable',
		info,
		apply: quoted || !wrapInQuotes ? value : `"${value}"`
	}));
}

function getValueRange(
	parsed: ParsedLine,
	pos: number,
	lineText: string,
	lineStart: number
): { from: number; to: number } {
	let valueStart: number;
	if (parsed.quoteChar) {
		const quotePos = parsed.beforeCursor.lastIndexOf(parsed.quoteChar);
		valueStart = lineStart + quotePos + 1;
	} else {
		const equalsPos = parsed.beforeCursor.lastIndexOf('=');
		const afterEquals = parsed.beforeCursor.slice(equalsPos + 1).trimStart();
		valueStart =
			lineStart +
			equalsPos +
			1 +
			(parsed.beforeCursor.slice(equalsPos + 1).length - afterEquals.length);
	}

	let valueEnd: number;
	if (parsed.quoteChar) {
		const closingQuotePos = parsed.afterCursor.indexOf(parsed.quoteChar);
		valueEnd = closingQuotePos >= 0 ? pos + closingQuotePos : pos + parsed.afterCursor.length;
	} else {
		valueEnd = pos + parsed.afterCursor.trimEnd().length;
	}

	return { from: valueStart, to: valueEnd };
}

function completeKnownValue(
	parsed: ParsedLine,
	pos: number,
	lineText: string,
	lineStart: number,
	values: string[],
	info: string
): CompletionResult {
	const { from, to } = getValueRange(parsed, pos, lineText, lineStart);
	const prefix = parsed.valueBeforeCursor;
	const filtered = prefix ? values.filter((value) => value.startsWith(prefix)) : values;

	return {
		start: from,
		end: to,
		options: buildValueCompletionOptions(filtered, parsed.quoteChar != null, info, true)
	};
}

function parseLine(lineText: string, cursorOffset: number): ParsedLine {
	const beforeCursor = lineText.slice(0, cursorOffset);
	const afterCursor = lineText.slice(cursorOffset);

	const fieldMatch = beforeCursor.match(/^\s*(\w+)\s*=\s*(.*)$/);

	if (fieldMatch) {
		const fieldName = fieldMatch[1];
		const afterEquals = fieldMatch[2];

		const quoteMatch = afterEquals.match(/^(['"`])/);
		if (quoteMatch) {
			const quoteChar = quoteMatch[1];
			const insideQuote = afterEquals.slice(1);

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

function findCurrentTable(lines: string[], lineNumber: number): string | null {
	for (let i = lineNumber - 1; i >= 0; i--) {
		const match = lines[i].trim().match(/^\[(\w+)\]$/);
		if (match) {
			return `[${match[1]}]`;
		}
	}
	return null;
}

function getExistingTables(lines: string[]): Set<string> {
	const tables = new Set<string>();
	for (const line of lines) {
		const match = line.trim().match(/^\[(\w+)\]$/);
		if (match) {
			tables.add(`[${match[1]}]`);
		}
	}
	return tables;
}

function completeModelId(
	parsed: ParsedLine,
	pos: number,
	lineText: string,
	lineStart: number
): CompletionResult | null {
	const models = modelIds.val ?? [];
	const prefix = parsed.valueBeforeCursor;
	const { from: valueStart, to: valueEnd } = getValueRange(parsed, pos, lineText, lineStart);

	if (!prefix || !prefix.includes('/')) {
		const providers = Array.from(new Set(models.map((id) => id.split('/')[0])));
		const filtered = prefix ? providers.filter((p) => p.startsWith(prefix)) : providers;

		return {
			start: valueStart,
			end: valueEnd,
			options: filtered.map((provider) => ({
				label: provider,
				type: 'variable',
				info: 'provider',
				apply: parsed.quoteChar ? provider : `"${provider}"`
			}))
		};
	} else {
		const filtered = models.filter((id) => id.startsWith(prefix));

		return {
			start: valueStart,
			end: valueEnd,
			options: filtered.map((id) => ({
				label: id,
				type: 'variable',
				info: 'provider/model',
				apply: parsed.quoteChar ? id : `"${id}"`
			}))
		};
	}
}

function completeBooleanValue(
	parsed: ParsedLine,
	pos: number,
	lineText: string,
	lineStart: number
): CompletionResult | null {
	const { from, to } = getValueRange(parsed, pos, lineText, lineStart);
	const prefix = parsed.valueBeforeCursor;
	const filtered = prefix
		? BOOLEAN_FIELD_VALUES.filter((value) => value.startsWith(prefix))
		: BOOLEAN_FIELD_VALUES;

	return {
		start: from,
		end: to,
		options: buildValueCompletionOptions(filtered, parsed.quoteChar != null, 'boolean', false)
	};
}

function completeModelFieldValue(
	parsed: ParsedLine,
	pos: number,
	lineText: string,
	lineStart: number
): CompletionResult | null {
	if (!parsed.fieldName) return null;

	if (MODEL_ID_FIELDS.has(parsed.fieldName)) {
		return completeModelId(parsed, pos, lineText, lineStart);
	}

	if (BOOLEAN_FIELDS.has(parsed.fieldName)) {
		return completeBooleanValue(parsed, pos, lineText, lineStart);
	}

	const stringValues = STRING_VALUE_FIELDS.get(parsed.fieldName);
	if (stringValues != null) {
		return completeKnownValue(parsed, pos, lineText, lineStart, stringValues, parsed.fieldName);
	}

	return null;
}

function completeFieldName(
	fields: string[],
	pos: number,
	lineText: string,
	lineStart: number
): CompletionResult | null {
	const beforeCursor = lineText.slice(0, pos - lineStart);
	const wordMatch = beforeCursor.match(/(\w*)$/);
	if (!wordMatch) return null;
	const word = wordMatch[1];

	return {
		start: pos - word.length,
		end: pos,
		options: fields.map((f) => ({
			label: f,
			type: 'variable'
		}))
	};
}

function hasFieldsInCurrentTable(
	lines: string[],
	lineNumber: number,
	currentTable: string | null
): boolean {
	if (!currentTable) return false;

	let foundTableHeader = false;
	for (let i = 0; i < lineNumber; i++) {
		const text = lines[i].trim();

		if (text === currentTable) {
			foundTableHeader = true;
			continue;
		}

		if (foundTableHeader && text.match(/^\[\w+\]$/)) {
			break;
		}

		if (foundTableHeader && text.match(/^\w+\s*=/)) {
			return true;
		}
	}

	return false;
}

function completeTableHeader(
	pos: number,
	lines: string[],
	lineNumber: number
): CompletionResult | null {
	const lineText = lines[lineNumber];
	const beforeCursor = lineText.slice(
		0,
		pos - lines.slice(0, lineNumber).join('\n').length - (lineNumber > 0 ? 1 : 0)
	);
	const lineStart = lines.slice(0, lineNumber).join('\n').length + (lineNumber > 0 ? 1 : 0);

	const existingTables = getExistingTables(lines);
	const currentTable = findCurrentTable(lines, lineNumber);

	const tableMatch = beforeCursor.match(/^(\s*)\[(\w*)$/);
	if (tableMatch) {
		const indent = tableMatch[1];
		const missingHeaders = TOML_TABLE_HEADERS.filter((h) => !existingTables.has(h));

		const afterCursor = lineText.slice(beforeCursor.length);
		const closingBracket = afterCursor.indexOf(']');
		const to = closingBracket >= 0 ? pos + closingBracket + 1 : pos;

		return {
			start: lineStart + indent.length,
			end: to,
			options: missingHeaders.map((h) => ({
				label: h,
				type: 'variable',
				info: 'TOML table header'
			}))
		};
	}

	if (/^\s*$/.test(beforeCursor)) {
		const hasFields = hasFieldsInCurrentTable(lines, lineNumber, currentTable);
		const shouldSuggestTables = !currentTable || hasFields;

		if (shouldSuggestTables) {
			const missingHeaders = TOML_TABLE_HEADERS.filter((h) => !existingTables.has(h));
			if (missingHeaders.length > 0) {
				return {
					start: lineStart,
					end: pos,
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

export function tomlCompletion(input: CompletionInput): CompletionResult | null {
	const { text, pos } = input;
	const lines = text.split('\n');

	let charCount = 0;
	let lineNumber = 0;
	for (let i = 0; i < lines.length; i++) {
		if (charCount + lines[i].length >= pos) {
			lineNumber = i;
			break;
		}
		charCount += lines[i].length + 1;
	}
	if (lineNumber >= lines.length) lineNumber = lines.length - 1;

	const lineText = lines[lineNumber];
	const lineStart = lines.slice(0, lineNumber).join('\n').length + (lineNumber > 0 ? 1 : 0);
	const cursorOffset = pos - lineStart;
	const parsed = parseLine(lineText, cursorOffset);

	if (parsed.inValue && parsed.fieldName) {
		return completeModelFieldValue(parsed, pos, lineText, lineStart);
	}

	const tableResult = completeTableHeader(pos, lines, lineNumber);
	if (tableResult) return tableResult;

	const currentTable = findCurrentTable(lines, lineNumber);

	if (!currentTable) {
		return completeFieldName(TOP_LEVEL_FIELDS, pos, lineText, lineStart);
	} else if (currentTable === '[capability]') {
		return completeFieldName(CAPABILITY_FIELDS, pos, lineText, lineStart);
	} else if (currentTable === '[parameter]') {
		return completeFieldName(PARAMETER_FIELDS, pos, lineText, lineStart);
	} else if (currentTable === '[media_gen]') {
		return completeFieldName(MEDIA_GEN_FIELDS, pos, lineText, lineStart);
	}

	return null;
}
