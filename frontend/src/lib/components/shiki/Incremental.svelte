<script lang="ts">
	import { preference } from '$lib/preference/index.svelte';
	import { getHighlighter, getThemeName, getThemeStyle } from './shiki';
	import Monochrome from './Monochrome.svelte';
	import { buildTokenHtml } from './incremental';
	import { ShikiStreamTokenizer } from '@shikijs/stream';
	import type { ShikiStreamTokenizerEnqueueResult } from '@shikijs/stream';
	import type { ThemedToken } from 'shiki';

	let {
		text = '',
		lang = 'text'
	}: {
		text?: string;
		lang?: string;
	} = $props();

	let themeName = $derived(getThemeName(preference.value.theme));
	let themeStyle = $derived(getThemeStyle(preference.value.theme));

	let lines = $state<string[]>([]);
	let currentLineSpans = $state('');
	let tokenizer = $state<ShikiStreamTokenizer | null>(null);
	let prevPushedLength = $state(0);

	$effect(() => {
		if (lang === 'text') return;
		let stopped = false;

		lines = [];
		currentLineSpans = '';
		prevPushedLength = 0;
		tokenizer = null;

		(async () => {
			if (stopped) return;

			const highlighter = await getHighlighter(lang, themeName);
			if (stopped) return;

			tokenizer = new ShikiStreamTokenizer({
				highlighter,
				lang,
				theme: themeName
			});
		})();

		return () => {
			stopped = true;
		};
	});

	function processStable(tokens: ThemedToken[]) {
		for (const t of tokens) {
			const parts = t.content.split('\n');
			for (let i = 0; i < parts.length; i++) {
				if (i > 0) {
					lines.push(currentLineSpans);
					currentLineSpans = '';
				}
				if (parts[i].length > 0) {
					currentLineSpans += buildTokenHtml({ ...t, content: parts[i] });
				}
			}
		}
	}

	$effect(() => {
		if (!tokenizer) return;

		if (text.length < prevPushedLength) {
			lines = [];
			currentLineSpans = '';
			prevPushedLength = 0;
			return;
		}

		const remaining = text.slice(prevPushedLength);
		if (remaining.length === 0) return;

		prevPushedLength = text.length;
		tokenizer.enqueue(remaining).then((result: ShikiStreamTokenizerEnqueueResult) => {
			processStable(result.stable);
		});
	});
</script>

{#if lines.length == 0}
	<Monochrome {text} />
{:else}
	<pre class="shiki {themeName}" style={themeStyle}><code
			>{#each lines as line}<div
					class="line min-h-6">{@html line}</div>{/each}{#if currentLineSpans}<div
					class="line min-h-6">{@html currentLineSpans}</div>{/if}</code
		></pre>
{/if}
