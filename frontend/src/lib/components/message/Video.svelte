<script lang="ts">
	import { onDestroy } from 'svelte';
	import { download } from '$lib/api/files.svelte';
	import { Download } from '@lucide/svelte';
	import { t } from 'svelte-intl-precompile';

	let { id, name }: { id: number; name?: string } = $props();

	let src = $state<string | undefined>(undefined);
	let error = $state(false);
	let isDownloading = $state(false);

	async function downloadVideo() {
		if (isDownloading) return;
		isDownloading = true;

		const src = await download(id);
		if (!src) {
			isDownloading = false;
			return;
		}

		const link = document.createElement('a');
		link.href = src;
		link.download = name ?? `video-${id}`;
		link.click();
		window.URL.revokeObjectURL(src);
		isDownloading = false;
	}

	function setSrc(newSrc: string) {
		if (src !== undefined) window.URL.revokeObjectURL(src);
		src = newSrc;
	}

	$effect(() => {
		download(id).then((url) => {
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
		{$t('chat.failed_load_video')}
	</div>
{:else if src}
	<div class="my-2 flex justify-center">
		<div class="group relative w-full max-w-3xl">
			<video
				{src}
				controls
				preload="metadata"
				class="h-auto max-h-[min(30rem,85vw,70vh)] w-full rounded-lg border border-border"
			>
				<track kind="captions" />
			</video>
			{#if name}
				<div class="absolute top-2 left-2 rounded bg-black/60 px-2 py-1 text-xs text-white">
					{name}
				</div>
			{/if}
			<div
				onclick={downloadVideo}
				onkeydown={(e) => {
					if (e.key === 'Enter' || e.key === ' ') {
						e.preventDefault();
						downloadVideo();
					}
				}}
				role="button"
				tabindex="0"
				aria-label="download video"
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
		{$t('chat.loading_video')}
	</div>
{/if}
