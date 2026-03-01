<script lang="ts">
	import type { ImageNode } from '../parser/types';
	import { BookX } from '@lucide/svelte';

	let { node }: { node: ImageNode } = $props();

	const url = $derived(node.url);
	const alt = $derived(node.alt);

	let errored = $state(false);
</script>

{#if errored}
	<div
		class="flex h-50 w-60 flex-col items-center justify-center rounded-md border border-outline text-lg"
	>
		<BookX class="h-10 w-10" />
		<span class="mt-1">Image not found</span>
	</div>
{:else}
	<img
		src={url}
		{alt}
		style="max-width: 100%;"
		onerror={() => (errored = true)}
		class="max-h-[70vh]"
	/>
{/if}
