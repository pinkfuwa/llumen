<script>
	import { getThemeName, getThemeStyle } from './shiki';
	import { highlight } from './highlight';
	import { preference } from '$lib/preference/index.svelte';
	import Monochrome from './Monochrome.svelte';

	let { lang = 'text', text = '', monochrome = false } = $props();

	let themeName = $derived(getThemeName(preference.value.theme));
	let themeStyle = $derived(getThemeStyle(preference.value.theme));

	let monochromeInner = $derived(lang == 'text' || monochrome);
</script>

{#if text.trim().length != 0}
	<div
		class="border-radius-md overflow-x-auto rounded-md border border-border p-2"
		style={themeStyle}
	>
		{#if monochromeInner}
			<Monochrome {text} />
		{:else}
			{#await highlight(text, lang, themeName)}
				<Monochrome {text} />
			{:then value}
				{@html value}
			{:catch}
				<Monochrome {text} />
			{/await}
		{/if}
	</div>
{/if}
