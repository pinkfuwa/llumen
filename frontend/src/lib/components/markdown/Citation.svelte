<script lang="ts">
	import { ClipboardCopy, SearchCheck, Search } from '@lucide/svelte';
	import { copy } from '$lib/copy';
	import Code from '../shiki/Code.svelte';
	import Root from './Root.svelte';

	interface Props {
		raw: string;
		url?: string;
		title?: string;
		favicon?: string;
		authoritative?: boolean;
	}

	let { raw, url, title, favicon, authoritative }: Props = $props();
</script>

<a
	class="mr-2 inline-flex flex-row items-center space-x-2 rounded-full bg-primary p-2 px-3 py-[2px] hover:bg-hover"
	href={url}
	target="_blank"
>
	{#if authoritative}
		<SearchCheck class="h-5 w-5" />
	{:else}
		<Search class="h-5 w-5" />
	{/if}
	{#if favicon}
		<img src={favicon} class="h-5 w-5" alt="icon" />
	{/if}
	<div>{title}</div>
</a>
