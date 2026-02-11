<script lang="ts">
	import Citation from '$lib/components/markdown/Citation.svelte';
	import type { UrlCitation } from '$lib/api/types';

	let { citations }: { citations: UrlCitation[] } = $props();

	const titleFromUrl = (url: string) => {
		try {
			return new URL(url).hostname;
		} catch {
			return url;
		}
	};
</script>

{#if citations.length}
	<div class="flex flex-wrap gap-2 pt-2">
		{#each citations as citation}
			<Citation
				raw={citation.title ?? citation.url}
				title={citation.title ?? titleFromUrl(citation.url)}
				url={citation.url}
				favicon={citation.favicon ?? ''}
				authoritative={false}
			/>
		{/each}
	</div>
{/if}
