<script lang="ts">
	let { id }: { id: number } = $props();

	import { useMessage } from '$lib/api/message';
	import { _ } from 'svelte-i18n';
	import Page from './Page.svelte';

	const { data } = useMessage(id);

	function scrollToBottom(node: HTMLElement) {
		node.scroll({ top: node.scrollHeight, behavior: 'smooth' });
	}
</script>

<div class="space-y-2">
	{#each $data as page}
		<Page entry={page} />
	{/each}
	{#key id}
		<div use:scrollToBottom></div>
	{/key}
</div>
