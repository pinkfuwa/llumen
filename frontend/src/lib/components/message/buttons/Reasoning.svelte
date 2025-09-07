<script lang="ts">
	import { slide } from 'svelte/transition';
	const { content }: { content: string } = $props();

	let showReasoning = $state(false);

	let lines = $derived(content.split('\n'));

	let contentTruncated = $derived(content.replaceAll('\n', ' '));
</script>

<button
	onclick={() => (showReasoning = !showReasoning)}
	class="w-full text-left"
	in:slide={{ duration: 180, axis: 'y' }}
>
	{#if showReasoning}
		<div>
			{#each lines as line}
				<p>{line}</p>
			{/each}
		</div>
	{:else}
		<div class="max-w-80 truncate text-outline" in:slide={{ duration: 180, axis: 'y' }}>
			Reasoning:&nbsp;{contentTruncated}
		</div>
	{/if}
</button>
