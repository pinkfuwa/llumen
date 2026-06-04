<script lang="ts">
	import { getThemeName, getThemeStyle } from './shiki';
	import { highlight } from './highlight';
	import { preference } from '$lib/preference/index.svelte';
	import Monochrome from './Monochrome.svelte';
	import Incremental from './Incremental.svelte';

	let {
		lang = 'text',
		text = '',
		incremental = false
	}: { lang?: string; text?: string; incremental?: boolean } = $props();

	let themeName = $derived(getThemeName(preference.value.theme));
	let themeStyle = $derived(getThemeStyle(preference.value.theme));

	let finalHtml = $state<string | null>(null);

	$effect(() => {
		if (lang !== 'text' && !incremental && text.trim().length > 0 && finalHtml === null) {
			highlight(text, lang, themeName).then((html) => {
				finalHtml = html;
			});
		}
	});
</script>

{#if text.trim().length != 0}
	<div
		class="border-radius-md overflow-x-auto rounded-md border border-border p-2"
		style={themeStyle}
	>
		{#if lang === 'text'}
			<Monochrome {text} />
		{:else if incremental || finalHtml === null}
			<Incremental {text} {lang} {incremental} />
		{:else}
			{@html finalHtml}
		{/if}
	</div>
{/if}
