<script>
	import { getThemeStyle } from './shiki';
	import { highlight } from './highlight';
	import { isLightTheme } from '$lib/preference';
	import Monochrome from './Monochrome.svelte';

	let { lang = 'text', text = '', monochrome = false } = $props();

	let themeStyle = $derived(getThemeStyle($isLightTheme));

	let monochromeInner = $derived(lang == 'text' || monochrome);
</script>

{#if text.trim().length != 0}
	<div
		class="border-radius-md overflow-x-auto rounded-md border border-outline p-2"
		style={themeStyle}
	>
		{#if monochromeInner}
			<Monochrome {text} />
		{:else}
			{#await highlight(text, lang, $isLightTheme ? 'light' : 'dark')}
				<Monochrome {text} />
			{:then value}
				{@html value}
			{:catch}
				<Monochrome {text} />
			{/await}
		{/if}
	</div>
{/if}
