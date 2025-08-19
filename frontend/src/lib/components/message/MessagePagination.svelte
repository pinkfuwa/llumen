<script lang="ts">
	let { id }: { id: number } = $props();

	import { useMessage } from '$lib/api/message';
	import { _ } from 'svelte-i18n';
	import Page from './Page.svelte';

	const { data } = useMessage(id);
</script>

{#each $data as page}
	{#key page.no}
		<Page entry={page} />
	{/key}
{/each}
