<script>
	import { codeToHtml, getThemeName, getThemeStyle } from './shiki';
	import { isLightTheme } from '$lib/preference';

	let { lang = 'typescript', text = '', monochrome = false } = $props();

	let themeName = $derived(getThemeName($isLightTheme));
	let themeStyle = $derived(getThemeStyle($isLightTheme));
</script>

<div
	class="border-radius-md overflow-x-auto rounded-md border border-outline p-2"
	style={themeStyle}
>
	{#if monochrome}
		<pre class="shiki {themeName}" style={themeStyle}><code
				>{#each text.split('\n') as line}<div class="line min-h-6"><span>{line}</span
						></div>{/each}</code
			></pre>
	{:else}
		{#await codeToHtml(text, { lang, isLight: $isLightTheme })}
			<pre class="shiki {themeName}" style={themeStyle}><code
					>{#each text.split('\n') as line}<div class="line min-h-6"><span>{line}</span
							></div>{/each}</code
				></pre>
		{:then value}
			{@html value}
		{:catch}
			<pre class="shiki {themeName}" style={themeStyle}><code
					>{#each text.split('\n') as line}<div class="line min-h-6"><span>{line}</span
							></div>{/each}</code
				></pre>
		{/await}
	{/if}
</div>
