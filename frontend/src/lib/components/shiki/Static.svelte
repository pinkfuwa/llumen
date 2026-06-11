<script lang="ts">
	import { getThemeName } from './shiki';
	import { highlight } from './highlight';
	import { preference } from '$lib/preference/index.svelte';
	import Monochrome from './Monochrome.svelte';

	let { lang = 'text', text = '' }: { lang?: string; text?: string } = $props();

	let themeName = $derived(getThemeName(preference.value.theme));

	let finalHtml = $state<string | null>(null);

	$effect(() => {
		finalHtml = null;

		if (lang === 'text' || text.trim().length === 0) return;

		let stopped = false;

		highlight(text, lang, themeName).then((html) => {
			if (!stopped) finalHtml = html;
		});

		return () => {
			stopped = true;
		};
	});
</script>

{#if finalHtml === null}
	<Monochrome {text} />
{:else}
	{@html finalHtml}
{/if}
