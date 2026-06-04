<script lang="ts">
	import { preference } from '$lib/preference/index.svelte';
	import { getThemeName, getThemeStyle } from './shiki';
	import Monochrome from './Monochrome.svelte';
	import { buildTokenHtml } from './incremental';
	import type { BundledLanguage } from './shiki.bundle';

	let {
		text = '',
		lang = 'text',
		incremental = true
	}: {
		text?: string;
		lang?: string;
		incremental?: boolean;
	} = $props();

	let themeName = $derived(getThemeName(preference.value.theme));
	let themeStyle = $derived(getThemeStyle(preference.value.theme));

	let ready = $state(false);
	let lines = $state<string[]>([]);
	let currentLineSpans = $state('');
	let tokenizer: any = null;

	let prevPushedLength = 0;

	$effect(() => {
		if (lang === 'text') return;
		let stopped = false;

		(async () => {
			const [bundle, streamModule] = await Promise.all([
				import('./shiki.bundle'),
				import('@shikijs/stream')
			]);
			if (stopped) return;

			const h = await bundle.createHighlighter({
				themes: [themeName],
				langs: []
			});
			if (stopped) return;

			if (!h.getLoadedLanguages().includes(lang)) {
				await h.loadLanguage(lang as BundledLanguage);
			}
			if (stopped) return;

			tokenizer = new streamModule.ShikiStreamTokenizer({
				highlighter: h,
				lang,
				theme: themeName
			});
			ready = true;
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
					lines = [...lines, currentLineSpans];
					currentLineSpans = '';
				}
				if (parts[i].length > 0) {
					currentLineSpans += buildTokenHtml({ ...t, content: parts[i] });
				}
			}
		}
	}

	$effect(() => {
		if (!ready || !tokenizer || !incremental) return;

		const remaining = text.slice(prevPushedLength);
		if (remaining.length === 0) return;

		prevPushedLength = text.length;
		tokenizer.enqueue(remaining).then((result: any) => {
			processStable(result.stable);
		});
	});
</script>

{#if !ready}
	<Monochrome {text} />
{:else}
	<pre class="shiki {themeName}" style={themeStyle}><code
			>{#each lines as line, i (i)}<div
					class="line min-h-6">{@html line}</div>{/each}{#if currentLineSpans}<div
					class="line min-h-6">{@html currentLineSpans}</div>{/if}</code
		></pre>
{/if}
