<script>
	import { ClipboardCopy } from '@lucide/svelte';
	import { theme } from '$lib/store';
	import { codeToHtml } from 'shiki/bundle/web';

	let { lang, text } = $props();

	let themeStyle =
		$theme == 'light'
			? 'background-color:#fff;color:#24292e'
			: 'background-color:#24292e;color:#e1e4e8';
	let themeName = $theme == 'light' ? 'github-light' : 'github-dark';
</script>

<div class="group relative">
	<button
		class="absolute top-0 right-0 z-10 m-1 hidden group-hover:block"
		onclick={() => {
			navigator.clipboard.writeText(text);
		}}
	>
		<ClipboardCopy class="h-10 w-10 rounded-md bg-background p-2 hover:bg-hover" />
	</button>
	<div
		class="border-radius-md overflow-x-scroll rounded-md border border-outline p-2"
		style={themeStyle}
	>
		{#await codeToHtml(text, { lang, theme: $theme == 'light' ? 'github-light' : 'github-dark' })}
			<pre class="shiki {themeName}" style={themeStyle}><code
					>{#each text.split('\n') as line}<div class="line"><span>{line}</span></div>{/each}</code
				></pre>
		{:then value}
			{@html value}
		{/await}
	</div>
</div>
