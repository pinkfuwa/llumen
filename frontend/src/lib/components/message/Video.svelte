<script lang="ts">
	import { onDestroy } from 'svelte';
	import { download } from '$lib/api/files.svelte';
	import { Download } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';

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
	<div class="border-border my-2 flex justify-center rounded-lg border p-4">
		{$_('chat.failed_load_video')}
	</div>
{:else if src}
	<div class="my-2 flex justify-center">
		<div class="group relative w-full max-w-3xl">
			<video
				{src}
				controls
				preload="metadata"
				class="border-border h-auto max-h-[min(30rem,85vw,70vh)] w-full rounded-lg border"
			>
				<track kind="captions" />
			</video>
			{#if name}
				<div class="absolute top-2 left-2 rounded bg-black/60 px-2 py-1 text-xs text-white">
					{name}
				</div>
			{/if}
			<button
				onclick={downloadVideo}
				disabled={isDownloading}
				aria-label="download video"
				class="visible absolute top-2 right-2 rounded-lg bg-secondary p-2 duration-150 hover:bg-primary hover:text-text-hover disabled:opacity-50 md:invisible md:group-hover:visible"
			>
				<Download class="h-5 w-5" />
			</button>
		</div>
	</div>
{:else}
	<div class="border-border my-2 flex justify-center rounded-lg border p-4">
		{$_('chat.loading_video')}
	</div>
{/if}
