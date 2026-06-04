import { EditorView } from '@codemirror/view';
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
import { tags as t } from '@lezer/highlight';
import type { Extension } from '@codemirror/state';

export const config = {
	name: 'vitesseDark',
	dark: true,
	background: '#1e1e1e',
	foreground: '#d4cfbf',
	selection: '#444444',
	cursor: '#d4cfbf',
	dropdownBackground: '#1e1e1e',
	dropdownBorder: '#444444',
	activeLine: '#44444444',
	lineNumber: '#888888',
	lineNumberActive: '#eeeeee',
	matchingBracket: '#444444',
	keyword: '#4d9375',
	storage: '#4d9375',
	variable: '#c2b36e',
	parameter: '#d4cfbf',
	function: '#a1b567',
	string: '#d48372',
	constant: '#e0a569',
	type: '#54b1bf',
	class: '#54b1bf',
	number: '#6394bf',
	comment: '#758575',
	heading: '#6394bf',
	invalid: '#a14f55',
	regexp: '#ab5e3f'
};

export const vitesseDarkTheme = EditorView.theme(
	{
		'&.cm-focused': {
			outline: 'none'
		},
		'.cm-line': {
			fontSize: '16px',
			lineHeight: '26.5px'
		},

		'&': {
			color: config.foreground,
			backgroundColor: config.background
		},

		'.cm-content': {
			caretColor: config.cursor,
			fontFamily:
				'var(--default-mono-font-family, ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace)'
		},

		'.cm-cursor, .cm-dropCursor': { borderLeftColor: config.cursor },
		'&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection':
			{ backgroundColor: config.selection },

		'.cm-panels': { backgroundColor: config.dropdownBackground, color: config.foreground },
		'.cm-panels.cm-panels-top': { borderBottom: '2px solid black' },
		'.cm-panels.cm-panels-bottom': { borderTop: '2px solid black' },

		'.cm-searchMatch': {
			backgroundColor: config.dropdownBackground,
			outline: `1px solid ${config.dropdownBorder}`
		},
		'.cm-searchMatch.cm-searchMatch-selected': {
			backgroundColor: config.selection
		},

		'.cm-activeLine': { backgroundColor: config.activeLine },
		'.cm-selectionMatch': { backgroundColor: config.selection },

		'&.cm-focused .cm-matchingBracket, &.cm-focused .cm-nonmatchingBracket': {
			backgroundColor: config.matchingBracket,
			outline: 'none'
		},

		'.cm-gutters': {
			backgroundColor: config.background,
			color: config.foreground,
			border: 'none'
		},
		'.cm-activeLineGutter': { backgroundColor: config.background },

		'.cm-lineNumbers .cm-gutterElement': { color: config.lineNumber },
		'.cm-lineNumbers .cm-activeLineGutter': { color: config.lineNumberActive },

		'.cm-foldPlaceholder': {
			backgroundColor: 'transparent',
			border: 'none',
			color: config.foreground
		},
		'.cm-tooltip': {
			border: `1px solid ${config.dropdownBorder}`,
			backgroundColor: config.dropdownBackground,
			color: config.foreground
		},
		'.cm-tooltip .cm-tooltip-arrow:before': {
			borderTopColor: 'transparent',
			borderBottomColor: 'transparent'
		},
		'.cm-tooltip .cm-tooltip-arrow:after': {
			borderTopColor: config.foreground,
			borderBottomColor: config.foreground
		},
		'.cm-tooltip-autocomplete': {
			'& > ul > li[aria-selected]': {
				background: config.selection,
				color: config.foreground
			}
		}
	},
	{ dark: config.dark }
);

export const vitesseDarkHighlightStyle = HighlightStyle.define([
	{ tag: t.keyword, color: config.keyword },
	{ tag: [t.name, t.deleted, t.character, t.macroName], color: config.variable },
	{ tag: [t.propertyName], color: config.function },
	{
		tag: [t.processingInstruction, t.string, t.inserted, t.special(t.string)],
		color: config.string
	},
	{ tag: [t.function(t.variableName), t.labelName], color: config.function },
	{ tag: [t.color, t.constant(t.name), t.standard(t.name)], color: config.constant },
	{ tag: [t.definition(t.name), t.separator], color: config.variable },
	{ tag: [t.className], color: config.class },
	{
		tag: [t.number, t.changed, t.annotation, t.modifier, t.self, t.namespace],
		color: config.number
	},
	{ tag: [t.typeName], color: config.type, fontStyle: config.type },
	{ tag: [t.operator, t.operatorKeyword], color: config.keyword },
	{ tag: [t.url, t.escape, t.regexp, t.link], color: config.regexp },
	{ tag: [t.meta, t.comment], color: config.comment },
	{ tag: t.strong, fontWeight: 'bold' },
	{ tag: t.emphasis, fontStyle: 'italic' },
	{ tag: t.link, textDecoration: 'underline' },
	{ tag: t.heading, fontWeight: 'bold', color: config.heading },
	{ tag: [t.atom, t.bool, t.special(t.variableName)], color: config.variable },
	{ tag: t.invalid, color: config.invalid },
	{ tag: t.strikethrough, textDecoration: 'line-through' }
]);

export default [vitesseDarkTheme, syntaxHighlighting(vitesseDarkHighlightStyle)] as Extension[];
