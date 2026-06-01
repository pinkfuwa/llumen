<script>
	import { getThemeName, getThemeStyle } from './shiki';
	import { highlight } from './highlight';
	import { theme } from '$lib/preference';
	import Monochrome from './Monochrome.svelte';
	import { derived } from 'svelte/store';

	let { lang = 'text', text = '', monochrome = false } = $props();

	let themeName = derived(theme, getThemeName);

	let themeStyle = derived(theme, getThemeStyle);

	let monochromeInner = $derived(lang == 'text' || monochrome);
</script>

{#if text.trim().length != 0}
	<div
		class="border-radius-md overflow-x-auto rounded-md border border-border p-2"
		style={$themeStyle}
	>
		{#if monochromeInner}
			<Monochrome {text} />
		{:else}
			{#await highlight(text, lang, $themeName)}
				<Monochrome {text} />
			{:then value}
				{@html value}
			{:catch}
				<Monochrome {text} />
			{/await}
		{/if}
	</div>
{/if}
