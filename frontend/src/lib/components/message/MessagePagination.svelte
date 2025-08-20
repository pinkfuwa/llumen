<script lang="ts">
	let { id }: { id: number } = $props();

	import { handleServerSideMessage, useMessage } from '$lib/api/message';
	import { _ } from 'svelte-i18n';
	import Page from './Page.svelte';

	const { data } = useMessage(id);

	handleServerSideMessage(id);
</script>

{#each $data as page}
	{#key page.no}
		<Page entry={page} />
	{/key}
{/each}
