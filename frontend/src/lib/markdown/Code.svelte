<script>
	import { ClipboardCopy } from '@lucide/svelte';
	import { theme } from '$lib/store';
	import { codeToHtml } from 'shiki/bundle/web';
	import CopyHint from './CopyHint.svelte';

	let { lang, text } = $props();

	let themeStyle =
		$theme == 'light'
			? 'background-color:#fff;color:#24292e'
			: 'background-color:#24292e;color:#e1e4e8';
	let themeName = $theme == 'light' ? 'github-light' : 'github-dark';

	let copied = $state(false);

	function copy() {
		navigator.clipboard.writeText(text);
		copied = true;
		setTimeout(() => {
			copied = false;
		}, 800);
	}
</script>

<div class="group/codeblock relative">
	{#if text.split('\n').length > 1}
		<button
			class="absolute top-0 right-0 z-10 m-1 hidden group-hover/codeblock:block"
			onclick={copy}
		>
			<ClipboardCopy class="h-10 w-10 rounded-md bg-background p-2 hover:bg-hover" />
		</button>
	{/if}
	<div
		class="border-radius-md overflow-x-auto rounded-md border border-outline p-2"
		style={themeStyle}
	>
		{#await codeToHtml(text, { lang, theme: $theme == 'light' ? 'github-light' : 'github-dark' })}
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
	</div>
</div>

{#if copied}
	<CopyHint />
{/if}
