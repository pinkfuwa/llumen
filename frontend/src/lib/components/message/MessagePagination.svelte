<script lang="ts">
	let { id }: { id: number } = $props();

	import { addSSEHandler, startSSE, useMessage } from '$lib/api/message';
	import Page from './Page.svelte';
	import MessageStream from './MessageStream.svelte';
	import { useRoomStreamingState } from '$lib/api/chatroom';
	import {
		type MessagePaginateRespChunk,
		type MessagePaginateRespList,
		MessagePaginateRespRole
	} from '$lib/api/types';
	import { SetInfiniteQueryData } from '$lib/api/state';

	const { data } = useMessage(id);

	let isStreaming = $derived(useRoomStreamingState(id));

	let chunks = $state<MessagePaginateRespChunk[]>([]);
	startSSE(id);

	addSSEHandler('message_end', (data) => {
		SetInfiniteQueryData<MessagePaginateRespList>({
			key: ['messagePaginate', id.toString()],
			data: {
				id: data.id,
				role: MessagePaginateRespRole.Assistant,
				chunks
			}
		});
		isStreaming.set(false);
		chunks = [];
	});
</script>

{#if $isStreaming}
	<MessageStream bind:chunks />
{/if}

{#each $data as page}
	{#key page.no}
		<Page entry={page} />
	{/key}
{/each}
