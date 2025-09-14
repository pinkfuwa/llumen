<script>
	import { ClipboardCopy } from '@lucide/svelte';
	import { copy } from '$lib/copy';
	import Code from '../shiki/Code.svelte';
	import Root from './Root.svelte';

	let { lang, text, monochrome = false } = $props();
</script>

{#if lang == 'markdown'}
	<Root source={text} />
{:else}
	<div class="group/codeblock relative">
		{#if text.split('\n').length > 1}
			<button
				class="absolute top-0 right-0 z-10 m-1 hidden group-hover/codeblock:block"
				onclick={() => copy(text)}
			>
				<ClipboardCopy class="h-10 w-10 rounded-md bg-primary p-2 hover:bg-hover" />
			</button>
		{/if}
		<Code {lang} {text} {monochrome} />
	</div>
{/if}
