<script lang="ts">
	import { Globe } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import { Collapsible } from 'bits-ui';
	import type { UrlCitation } from '$lib/api/types';

	let { citations, open = $bindable(false) }: { citations: UrlCitation[]; open?: boolean } =
		$props();

	const titleFromUrl = (url: string) => {
		try {
			return new URL(url).hostname;
		} catch {
			return url;
		}
	};
</script>

{#if citations.length}
	<Collapsible.Root bind:open>
		<Collapsible.Trigger
			class="flex flex-row flex-nowrap items-center rounded p-2 duration-150 hover:bg-primary hover:text-text-hover"
		>
			<Globe class="mr-2" />
			<span>
				{$_('chat.sources')}
				<span class="ml-1 text-xs opacity-60">({citations.length})</span>
			</span>
		</Collapsible.Trigger>
		<Collapsible.Content
			class="py-2 slide-out-to-start-2 fade-in fade-out slide-in-from-top-2 data-[state=close]:animate-out data-[state=open]:animate-in"
		>
			<div class="flex flex-col gap-1.5 pl-3">
				{#each citations as citation}
					<a
						href={citation.url}
						target="_blank"
						class="flex flex-row items-center gap-2.5 rounded-lg border border-outline p-2.5 duration-150 hover:bg-primary hover:text-text-hover"
					>
						{#if citation.favicon}
							<img
								src={citation.favicon}
								alt=""
								class="h-4 w-4 shrink-0 rounded-sm"
								onerror={(e) => ((e.target as HTMLImageElement).style.display = 'none')}
							/>
						{/if}
						<div class="min-w-0 flex-1">
							<div class="truncate text-sm font-medium">
								{citation.title ?? titleFromUrl(citation.url)}
							</div>
							<div class="truncate text-xs opacity-50">
								{titleFromUrl(citation.url)}
							</div>
						</div>
					</a>
				{/each}
			</div>
		</Collapsible.Content>
	</Collapsible.Root>
{/if}
