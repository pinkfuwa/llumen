<script lang="ts">
	import { Brain } from '@lucide/svelte';
	import { slide } from 'svelte/transition';
	const { content }: { content: string } = $props();

	let showReasoning = $state(false);

	let lines = $derived(content.split('\n'));
</script>

<button
	onclick={() => (showReasoning = !showReasoning)}
	class="w-full border-l-6 border-primary py-1 pl-4 text-left"
>
	<div class="flex items-center">
		<Brain class="mr-2" /> Reasoning{#if !showReasoning}...{/if}
	</div>
	{#if showReasoning}
		<div
			class="mt-1"
			in:slide={{ duration: 180, axis: 'y' }}
			out:slide={{ duration: 180, axis: 'y' }}
		>
			{#each lines as line}
				<p>{line}</p>
			{/each}
		</div>
	{/if}
</button>
