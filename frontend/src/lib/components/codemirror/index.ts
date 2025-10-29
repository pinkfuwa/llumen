import { historyKeymap } from '@codemirror/commands';

import { StreamLanguage } from '@codemirror/language';
import { EditorView, keymap, type ViewUpdate } from '@codemirror/view';
import { toml } from '@codemirror/legacy-modes/mode/toml';
import githubLight from './github-light';
import githubDark from './github-dark';
import { get, type Readable, type Writable } from 'svelte/store';
import type { CompletionContext } from '@codemirror/autocomplete';
import { autocompletion, acceptCompletion } from '@codemirror/autocomplete';
import { minimalSetup } from 'codemirror';

const TOML_FIELDS = [
	'display_name',
	'model_id',
	'capability.image',
	'capability.audio',
	'capability.ocr',
	'capability.tool',
	'parameter.temperature',
	'parameter.repeat_penalty',
	'parameter.top_k',
	'parameter.top_p'
];

const TOML_TABLE_HEADERS = ['[capability]', '[parameter]'];

let modelIds: string[] = [];
let modelIdsFetched = false;

async function fetchModelIds() {
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

function tomlCompletion(context: CompletionContext) {
	const { state, pos } = context;
	const lineObj = state.doc.lineAt(pos);
	const lineText = lineObj.text;
	const before = lineText.slice(0, pos - lineObj.from);

	// Table header completion: at start of line or after blank line, or if line starts with '['
	if (/^\s*$/.test(before) || /^\s*\[/.test(before)) {
		return {
			from: lineObj.from,
			options: TOML_TABLE_HEADERS.map((h) => ({
				label: h,
				type: 'variable',
				info: 'TOML table header'
			})),
			validFor: /^\[.*$/
		};
	}

	// If editing model_id line, show modelIds
	if (/^\s*model_id\s*=/.test(lineText)) {
		const word = context.matchBefore(/\w+(\.\w+)?/);
		if (!word) return null;
		return {
			from: word.from,
			options: modelIds.map((id) => ({ label: id, type: 'variable' }))
		};
	}

	// Context-aware: if inside [capability] or [parameter] table, suggest only relevant keys
	const prevLines = [];
	for (let i = lineObj.number - 1; i >= 1 && i >= lineObj.number - 10; i--) {
		const prevLine = state.doc.line(i).text.trim();
		if (prevLine.startsWith('[') && prevLine.endsWith(']')) {
			prevLines.push(prevLine);
			break;
		}
	}
	const currentTable = prevLines.length ? prevLines[0] : null;

	let options: string[] = TOML_FIELDS;
	if (currentTable === '[capability]') {
		options = ['image', 'audio', 'ocr', 'tool'];
	} else if (currentTable === '[parameter]') {
		options = ['temperature', 'repeat_penalty', 'top_k', 'top_p'];
	}

	const word = context.matchBefore(/\w+(\.\w+)?/);
	if (!word) return null;

	return {
		from: word.from,
		options: options.map((f) => ({ label: f, type: 'variable' }))
	};
}

export default function useCodeMirror(option: {
	isLightTheme: boolean;
	value: Writable<string>;
	element: Readable<HTMLDivElement>;
	onDestroy: (callback: () => void) => void;
}) {
	const { isLightTheme, value, element, onDestroy } = option;

	const div = get(element);

	let onUpdate = EditorView.updateListener.of((v: ViewUpdate) => {
		if (!v.docChanged) return;
		const newValue = v.view.state.doc.toString();
		value.set(newValue);
	});

	if (!modelIdsFetched) {
		fetchModelIds();
	}

	const autocompleteTheme = EditorView.theme({
		'.cm-tooltip-autocomplete': {
			borderRadius: '0.5rem !important',
			border: '1px solid var(--color-outline, #e5e7eb) !important',
			background: 'var(--color-input, #fff) !important',
			color: 'var(--color-text, #24292e) !important',
			boxShadow: '0 4px 32px 0 rgba(0,0,0,0.08) !important',
			padding: '0.25rem 0 !important',
			fontSize: '0.95rem !important',
			zIndex: '1000 !important',
			position: 'relative !important',
			top: '100% !important',
			left: '0 !important',
			right: '0 !important',
			fontFamily:
				'var(--default-mono-font-family, ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace) !important',
			overflowX: 'hidden !important'
		},
		'.cm-tooltip-autocomplete > ul': {
			padding: '0 !important',
			listStyle: 'none !important' // Remove default list style
		},
		'.cm-tooltip-autocomplete li': {
			padding: '0.25rem 0.5rem !important', // px-2 py-3
			cursor: 'pointer !important',
			border: 'none !important',
			transition: 'background 0.15s, color 0.15s !important'
		}
	});

	let view = new EditorView({
		parent: div!,
		doc: get(value),
		extensions: [
			minimalSetup,
			keymap.of([
				...historyKeymap,
				{
					key: 'Tab',
					run: (view) => acceptCompletion(view),
					preventDefault: true
				}
			]),
			StreamLanguage.define(toml),
			isLightTheme ? githubLight : githubDark,
			onUpdate,
			autocompleteTheme,
			autocompletion({ override: [tomlCompletion] })
		]
	});

	onDestroy(() => {
		view.destroy();
	});
}
