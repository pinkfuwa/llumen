<script lang="ts">
	import type { ImageNode } from '../vendor/types';
	import { BookX } from '@lucide/svelte';

	let { node }: { node: ImageNode } = $props();

	const url = $derived(node.url);
	const alt = $derived(node.alt);

	let errored = $state(false);

	const imageContainerStyle =
		'flex h-50 w-60 flex-col items-center justify-center rounded-md border border-border text-lg';
</script>

{#if errored}
	<div class={imageContainerStyle}>
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
