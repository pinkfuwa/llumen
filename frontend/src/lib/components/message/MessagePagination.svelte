<script lang="ts">
	let { id }: { id: number } = $props();

	import { addSSEHandler, startSSE, useMessage as useMessages } from '$lib/api/message';
	import Page from './Page.svelte';
	import MessageStream from './MessageStream.svelte';
	import { updateRoomTitle } from '$lib/api/chatroom';

	const { data } = useMessages(id);

	startSSE(id);

	addSSEHandler('title', (data) => {
		updateRoomTitle(id, data.title);
	});
</script>

<MessageStream chat_id={id} />

{#each $data as page}
	{#key page.no}
		<Page entry={page} />
	{/key}
{/each}
