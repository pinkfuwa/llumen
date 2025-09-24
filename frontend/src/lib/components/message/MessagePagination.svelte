<script lang="ts">
	import type { ChatReadResp } from '$lib/api/types';
	import { addSSEHandler, startSSE, useMessage } from '$lib/api/message';
	import Page from './Page.svelte';
	import MessageStream from './MessageStream.svelte';
	import { updateRoomTitle } from '$lib/api/chatroom';

	// TODO: ChatReadResp is props drill, use globalCache
	let { id, room }: { id: number; room: ChatReadResp | undefined } = $props();

	const { data } = useMessage(id);

	startSSE(id);

	addSSEHandler('title', (data) => {
		updateRoomTitle(id, data.title);
	});
</script>

<MessageStream chat_id={id} />

{#each $data as page}
	{#key page.no}
		<Page entry={page} roomId={id} {room} />
	{/key}
{/each}
