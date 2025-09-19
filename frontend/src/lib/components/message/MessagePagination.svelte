<script lang="ts">
	let { id }: { id: number } = $props();

	import { startSSE, useMessage } from '$lib/api/message';
	import Page from './Page.svelte';
	import MessageStream from './MessageStream.svelte';

	const { data } = useMessage(id);

	startSSE(id);
</script>

<MessageStream chat_id={id} />

{#each $data as page}
	{#key page.no}
		<Page entry={page} />
	{/key}
{/each}
