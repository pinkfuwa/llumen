import { minimalSetup } from 'codemirror';
import { StreamLanguage } from '@codemirror/language';
import { EditorView, keymap, type ViewUpdate } from '@codemirror/view';
import { indentWithTab } from '@codemirror/commands';
import { toml } from '@codemirror/legacy-modes/mode/toml';
import githubLight from './github-light';
import githubDark from './github-dark';
import { get, type Readable, type Writable } from 'svelte/store';

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

	let view = new EditorView({
		parent: div!,
		doc: get(value),
		extensions: [
			minimalSetup,
			keymap.of([indentWithTab]),
			StreamLanguage.define(toml),
			isLightTheme ? githubLight : githubDark,
			onUpdate
		]
	});

	onDestroy(() => {
		view.destroy();
	});
}
