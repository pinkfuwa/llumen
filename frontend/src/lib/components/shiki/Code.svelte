<script>
	import { codeToHtml, getThemeStyle } from './shiki';
	import { isLightTheme } from '$lib/preference';
	import Monochrome from './Monochrome.svelte';
	import { get } from 'svelte/store';

	let { lang = 'bash', text = '', monochrome = false } = $props();

	let themeStyle = $derived(getThemeStyle($isLightTheme));
</script>

<div
	class="border-radius-md overflow-x-auto rounded-md border border-outline p-2"
	style={themeStyle}
>
	{#if monochrome}
		<Monochrome {text} />
	{:else}
		{#await codeToHtml(text, { lang, isLight: $isLightTheme })}
			<Monochrome {text} />
		{:then value}
			{@html value}
		{:catch}
			<Monochrome {text} />
		{/await}
	{/if}
</div>
