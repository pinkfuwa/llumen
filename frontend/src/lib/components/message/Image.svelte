<script lang="ts">
	import { onDestroy } from 'svelte';
	import { downloadCompressed, download } from '$lib/api/files.svelte';
	import { Download } from '@lucide/svelte';
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);

	let { id, name }: { id: number; name?: string } = $props();

	let src = $state<string | undefined>(undefined);
	let error = $state(false);
	let isDownloading = $state(false);

	async function downloadImage() {
		if (isDownloading) return;
		isDownloading = true;

		const src = await download(id);
		if (!src) return;

		const link = document.createElement('a');
		link.href = src;
		link.download = name ?? `image-${id}`;
		link.click();
		window.URL.revokeObjectURL(src);
		isDownloading = false;
	}

	function setSrc(newSrc: string) {
		if (src !== undefined) window.URL.revokeObjectURL(src);
		src = newSrc;
	}

	$effect(() => {
		downloadCompressed(id).then((url) => {
			if (url) setSrc(url);
			else error = true;
		});
	});

	onDestroy(() => {
		if (src !== undefined) window.URL.revokeObjectURL(src);
	});
</script>

{#if error}
	<div class="my-2 flex justify-center rounded-lg border border-border p-4">
		{m['chat.failed_load_image'](lang)}
	</div>
{:else if src}
	<div class="my-2 flex justify-center">
		<div class="group relative">
			<img
				{src}
				alt={m['chat.image_alt'](lang)}
				class="group relative h-auto max-h-[min(30rem,85vw,70vh)] max-w-full rounded-lg border border-border"
				loading="lazy"
			/>
			{#if name}
				<div class="absolute top-2 left-2 rounded bg-black/60 px-2 py-1 text-xs text-white">
					{name}
				</div>
			{/if}
			<div
				onclick={downloadImage}
				onkeydown={(e) => {
					if (e.key === 'Enter' || e.key === ' ') {
						e.preventDefault();
						downloadImage();
					}
				}}
				role="button"
				tabindex="0"
				aria-label="download image"
				class="visible absolute top-2 right-2 cursor-pointer rounded-lg bg-muted p-2 duration-150 hover:bg-interactive-hover md:invisible md:group-hover:visible{isDownloading
					? ' opacity-50'
					: ''}"
			>
				<Download class="h-5 w-5" />
			</div>
		</div>
	</div>
{:else}
	<div class="my-2 flex justify-center rounded-lg border border-border p-4">
		{m['chat.loading_image'](lang)}
	</div>
{/if}
