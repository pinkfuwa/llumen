<script lang="ts">
	import { preference } from '$lib/preference/index.svelte';
	import { bundle, getThemeName, getThemeStyle } from './shiki';
	import Monochrome from './Monochrome.svelte';
	import { buildTokenHtml } from './incremental';
	import { type BundledLanguage } from './shiki.bundle';
	import { ShikiStreamTokenizer } from '@shikijs/stream';

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
	let tokenizer: any = null;

	let prevPushedLength = 0;

	$effect(() => {
		if (lang === 'text') return;
		let stopped = false;

		lines = [];
		currentLineSpans = '';
		prevPushedLength = 0;
		tokenizer = null;

		(async () => {
			if (stopped) return;

			const localbundle = await bundle;

			const h = await localbundle.createHighlighter({
				themes: [themeName],
				langs: []
			});
			if (stopped) return;

			if (!h.getLoadedLanguages().includes(lang)) {
				await h.loadLanguage(lang as BundledLanguage);
			}
			if (stopped) return;

			tokenizer = new ShikiStreamTokenizer({
				highlighter: h,
				lang,
				theme: themeName
			});
		})();

		return () => {
			stopped = true;
		};
	});

	function processStable(tokens: any[]) {
		for (const t of tokens) {
			const parts = t.content.split('\n');
			for (let i = 0; i < parts.length; i++) {
				if (i > 0) {
					lines.push(...currentLineSpans);
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

		const remaining = text.slice(prevPushedLength);
		if (remaining.length === 0) return;

		prevPushedLength = text.length;
		tokenizer.enqueue(remaining).then((result: any) => {
			processStable(result.stable);
		});
	});
</script>

{#if lines.length == 0}
	<Monochrome {text} />
{:else}
	<pre class="shiki {themeName}" style={themeStyle}><code
			>{#each lines as line, i (i)}<div
					class="line min-h-6">{@html line}</div>{/each}{#if currentLineSpans}<div
					class="line min-h-6">{@html currentLineSpans}</div>{/if}</code
		></pre>
{/if}
