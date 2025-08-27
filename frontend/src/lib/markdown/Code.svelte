<script>
	import { ClipboardCopy } from '@lucide/svelte';
	import { codeToHtml } from 'shiki/bundle/web';
	import { isLightTheme } from '$lib/preference';
	import { copy } from '$lib/copy';

	let { lang, text, monochrome = false } = $props();

	let themeStyle = $isLightTheme
		? 'background-color:#fff;color:#24292e'
		: 'background-color:#24292e;color:#e1e4e8';
	let themeName = $isLightTheme ? 'github-light' : 'github-dark';
</script>

<div class="group/codeblock relative">
	{#if text.split('\n').length > 1}
		<button
			class="absolute top-0 right-0 z-10 m-1 hidden group-hover/codeblock:block"
			onclick={() => copy(text)}
		>
			<ClipboardCopy class="h-10 w-10 rounded-md bg-primary p-2 hover:bg-hover" />
		</button>
	{/if}
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
			{#await codeToHtml(text, { lang, theme: themeName })}
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
</div>
