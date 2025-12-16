<script lang="ts">
	import type { ImageToken } from './lexer';
	import { BookX } from '@lucide/svelte';

	let { token }: { token: ImageToken } = $props();

	const url = $derived(token.url);
	const alt = $derived(token.alt);
	const title = $derived(token.title);

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
		alt=""
		style="max-width: 100%;"
		onerror={() => (errored = true)}
		class="max-h-[70vh]"
	/>
{/if}
