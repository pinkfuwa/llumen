<script lang="ts">
	import { slide } from 'svelte/transition';
	import type { TokensList } from 'marked';
	import Parser from '$lib/markdown/Parser.svelte';

	const { list }: { list: TokensList[] } = $props();

	let reasoningContent = $derived(
		list
			.slice(0, 3)
			.flat()
			.map((x) => x.raw)
			.join('')
	);

	let showReasoning = $state(false);
</script>

<button
	onclick={() => (showReasoning = !showReasoning)}
	class="w-full text-left"
	in:slide={{ duration: 180, axis: 'y' }}
>
	{#if showReasoning}
		<div>
			<Parser tokens={list} monochrome />
		</div>
	{:else}
		<div class="max-w-80 truncate text-outline" in:slide={{ duration: 180, axis: 'y' }}>
			Reasoning:&nbsp;{reasoningContent.replaceAll('\n', '&nbsp;&nbsp;')}
		</div>
	{/if}
</button>
