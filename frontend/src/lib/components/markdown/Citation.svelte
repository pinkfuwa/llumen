<script lang="ts">
	import { SearchCheck, Search } from '@lucide/svelte';

	interface Props {
		raw: string;
		url?: string;
		title?: string;
		favicon?: string;
		authoritative?: boolean | 'true' | 'false';
	}

	let { raw, url, title, favicon, authoritative }: Props = $props();

	let imageErrored = $state(false);
</script>

<a
	class="mr-2 inline-flex flex-row items-center space-x-2 overflow-hidden rounded-full border-b border-outline p-2 px-3 py-[2px] hover:bg-primary hover:text-text-hover"
	href={url}
	target="_blank"
>
	{#if authoritative == true || authoritative == 'true'}
		<SearchCheck class="h-5 w-5" />
	{:else}
		<Search class="h-5 w-5" />
	{/if}
	{#if favicon && !imageErrored}
		<img src={favicon} class="h-5 w-5" alt="icon" onerror={() => (imageErrored = true)} />
	{/if}
	<div>{title}</div>
</a>
