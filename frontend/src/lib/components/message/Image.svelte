<script lang="ts">
	import { onMount } from 'svelte';
	import { download } from '$lib/api/files';
	import { Download } from '@lucide/svelte';

	let { id }: { id: number } = $props();

	let src = $state<string | undefined>(undefined);
	let error = $state(false);

	async function downloadImage() {
		if (!src) return;
		const link = document.createElement('a');
		link.href = src;
		link.download = `image-${id}`;
		link.click();
	}

	// TODO: recording loading promise and cancel on next load.
	$effect(() => {
		download(id).then((url) => {
			if (url) {
				src = url;
			} else {
				error = true;
			}
		});
	});
</script>

{#if error}
	<div class="border-border my-2 flex justify-center rounded-lg border p-4">
		Failed to load image
	</div>
{:else if src}
	<div class="my-2 flex justify-center">
		<div class="group relative">
			<img
				{src}
				alt="Generated"
				class="border-border group relative h-auto max-h-[min(30rem,85vw,70vh)] max-w-full rounded-lg border"
				loading="lazy"
			/>
			<button
				onclick={downloadImage}
				aria-label="download image"
				class="visible absolute top-2 right-2 rounded-lg bg-secondary p-2 duration-150 hover:bg-primary hover:text-text-hover md:invisible md:group-hover:visible"
			>
				<Download class="h-5 w-5" />
			</button>
		</div>
	</div>
{:else}
	<div class="border-border my-2 flex justify-center rounded-lg border p-4">Loading image...</div>
{/if}
