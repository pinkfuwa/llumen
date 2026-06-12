<script lang="ts">
	import { preference } from '$lib/preference/index.svelte';
	import { tomlCompletion, type CompletionOption } from './completion';
	import Autocomplete from './Autocomplete.svelte';
	import { getThemeName, getThemeStyle } from '$lib/components/shiki/shiki';
	import type { BundledTheme, codeToHtml } from '$lib/components/shiki/shiki.bundle';

	let {
		value = $bindable('# defaultConfig'),
		onchange = undefined as ((val: string) => void) | undefined
	} = $props();

	let textarea = $state<HTMLTextAreaElement | null>(null);
	let overlay = $state<HTMLDivElement | null>(null);
	let html = $state('');
	let bundle = import('$lib/components/shiki/shiki.bundle');

	let showAutocomplete = $state(false);
	let completions = $state<CompletionOption[]>([]);
	let cursorX = $state(0);
	let cursorY = $state(0);
	let completionStart = $state(0);
	let completionEnd = $state(0);

	let themeName = $derived(getThemeName(preference.value.theme) as BundledTheme);
	let themeStyle = $derived(getThemeStyle(preference.value.theme));

	$effect(() => {
		let args: Parameters<typeof codeToHtml> = [value, { lang: 'toml', theme: themeName }];

		let stopped = false;

		bundle.then(async (m) => {
			let result = await m.codeToHtml(...args);
			if (!stopped) html = result;
		});

		return () => {
			stopped = true;
		};
	});

	$effect(() => {
		if (!onchange) return;
		onchange(value);
	});

	function getMirrorCoords() {
		const ta = textarea!;
		const pos = ta.selectionStart;
		const cs = getComputedStyle(ta);
		const borderLeft = parseFloat(cs.borderLeftWidth) || 0;
		const borderTop = parseFloat(cs.borderTopWidth) || 0;
		const lineH = parseFloat(cs.lineHeight) || 20;

		const mirror = document.createElement('div');
		const props = [
			'fontSize',
			'fontFamily',
			'fontWeight',
			'fontStyle',
			'fontVariant',
			'fontStretch',
			'lineHeight',
			'letterSpacing',
			'wordSpacing',
			'textIndent',
			'textTransform',
			'wordBreak',
			'overflowWrap',
			'tabSize',
			'direction',
			'paddingTop',
			'paddingRight',
			'paddingBottom',
			'paddingLeft',
			'borderTopWidth',
			'borderRightWidth',
			'borderBottomWidth',
			'borderLeftWidth',
			'boxSizing',
			'width',
			'minHeight'
		];
		for (const p of props) {
			(mirror.style as any)[p] = (cs as any)[p];
		}
		mirror.style.position = 'absolute';
		mirror.style.left = '-9999px';
		mirror.style.top = '0';
		mirror.style.overflow = 'hidden';
		mirror.style.whiteSpace = 'pre-wrap';
		mirror.style.wordWrap = 'break-word';
		document.body.appendChild(mirror);

		const before = value.substring(0, pos);
		const after = value.substring(pos);
		mirror.appendChild(document.createTextNode(before));
		const cursorSpan = document.createElement('span');
		cursorSpan.textContent = after || '.';
		mirror.appendChild(cursorSpan);

		const taRect = ta.getBoundingClientRect();
		const { offsetLeft, offsetTop } = cursorSpan;
		document.body.removeChild(mirror);
		return {
			x: taRect.left + borderLeft + offsetLeft - ta.scrollLeft,
			y: taRect.top + borderTop + offsetTop - ta.scrollTop,
			h: lineH
		};
	}

	function onInput() {
		if (!textarea) return;
		const pos = textarea.selectionStart;
		const result = tomlCompletion({ text: value, pos });
		if (result && result.options.length > 0) {
			completions = result.options;
			completionStart = result.start;
			completionEnd = result.end;
			const coords = getMirrorCoords();
			cursorX = coords.x;
			cursorY = coords.y + coords.h;
			showAutocomplete = true;
		} else {
			showAutocomplete = false;
		}
	}

	function onKeydown(e: KeyboardEvent) {
		if (!showAutocomplete) return;
		if (['ArrowUp', 'ArrowDown', 'Enter', 'Tab', 'Escape'].includes(e.key)) {
			e.preventDefault();
		}
	}

	function selectOption(opt: CompletionOption) {
		const apply = opt.apply ?? opt.label;
		const newValue = value.substring(0, completionStart) + apply + value.substring(completionEnd);
		value = newValue;
		showAutocomplete = false;
		requestAnimationFrame(() => {
			textarea?.focus();
			const newPos = completionStart + apply.length;
			textarea?.setSelectionRange(newPos, newPos);
		});
	}

	function closeAutocomplete() {
		showAutocomplete = false;
	}

	$effect(() => {
		const ta = textarea;
		const ov = overlay;
		if (!ta || !ov) return;
		function sync() {
			ov!.scrollTop = ta!.scrollTop;
			ov!.scrollLeft = ta!.scrollLeft;
		}
		ta.addEventListener('scroll', sync);
		return () => ta.removeEventListener('scroll', sync);
	});
</script>

<div class="relative grid w-full rounded-md border border-border" style={themeStyle}>
	<style>
		:global(.shiki) {
			background: transparent !important;
		}
		.editor-overlay :global(pre.shiki) {
			margin: 0;
			padding: 0;
			background: transparent !important;
		}
		.editor-overlay :global(pre.shiki code) {
			display: block;
		}
	</style>
	{#if html}
		<div
			bind:this={overlay}
			class="editor-overlay pointer-events-none col-start-1 row-start-1 overflow-hidden p-2 font-mono"
			style="tab-size:4"
		>
			{@html html}
		</div>
	{/if}
	<textarea
		bind:this={textarea}
		bind:value
		oninput={onInput}
		onkeydown={onKeydown}
		class="col-start-1 row-start-1 resize-none overflow-auto bg-transparent p-2 font-mono text-nowrap text-transparent caret-inherit focus:outline-none"
		style="tab-size:4;field-sizing:content"
		spellcheck="false"
	></textarea>
</div>

{#if showAutocomplete}
	<Autocomplete
		options={completions}
		x={cursorX}
		y={cursorY}
		onselect={selectOption}
		onclose={closeAutocomplete}
	/>
{/if}
