<script>
	import { ClipboardCopy } from '@lucide/svelte';
	import { copy } from '$lib/copy';
	import Code from '../shiki/Code.svelte';
	import { Markdown } from '.';

	let { lang, text, monochrome = false } = $props();
</script>

{#if lang == 'markdown'}
	<Markdown source={text} />
{:else}
	<div class="group/codeblock relative">
		{#if text.split('\n').length > 1}
			<button
				class="absolute top-0 right-0 m-1 duration-150 group-hover/codeblock:visible md:invisible"
				onclick={() => copy(text)}
			>
				<ClipboardCopy
					class="h-10 w-10 rounded-lg p-2 duration-150 hover:bg-primary hover:text-text-hover"
				/>
			</button>
			<Code {lang} {text} {monochrome} />
		{:else}
			<button onclick={() => copy(text)} class="w-full cursor-pointer">
				<Code {lang} {text} {monochrome} />
			</button>
		{/if}
	</div>
{/if}
