import { historyKeymap } from '@codemirror/commands';

import { StreamLanguage } from '@codemirror/language';
import { EditorView, keymap, type ViewUpdate } from '@codemirror/view';
import { toml } from '@codemirror/legacy-modes/mode/toml';
import githubLight from './github-light';
import githubDark from './github-dark';
import { get, type Readable, type Writable } from 'svelte/store';
import { autocompletion, acceptCompletion } from '@codemirror/autocomplete';
import { minimalSetup } from 'codemirror';

import { modelIdsFetched, fetchModelIds, tomlCompletion } from './completion';

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
		},
		'.cm-completionIcon-keyword::before': {
			content: 'none !important',
			display: 'none !important'
		},
		'.cm-completionIcon-variable::before': {
			content: 'none !important',
			display: 'none !important'
		},
		'.cm-completionIcon-table::before': {
			content: 'none !important',
			display: 'none !important'
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
