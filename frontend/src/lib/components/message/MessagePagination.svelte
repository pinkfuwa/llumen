<script lang="ts">
	import { getMessages, useSSEEffect, updateMessage } from '$lib/api/messageDirect.svelte';
	import {
		MessagePaginateRespRole as Role,
		type ChatReadResp,
		type MessagePaginateRespChunk,
		type MessagePaginateRespChunkKindFile,
		type MessagePaginateRespChunkKindText
	} from '$lib/api/types';
	import { dispatchError } from '$lib/error';
	import ResponseBox from './buttons/ResponseBox.svelte';
	import ResponseEdit from './buttons/ResponseEdit.svelte';
	import User from './buttons/User.svelte';
	import Chunks from './Chunks.svelte';
	import { page } from '$app/state';

	const { room }: { room: ChatReadResp | undefined } = $props();

	const chatId = $derived(parseInt(page.params.id));

	let { mutate } = updateMessage();

	useSSEEffect(() => chatId);

	function getTextFromChunks(chunks: MessagePaginateRespChunk[]) {
		return chunks
			.filter((x) => x.kind.t == 'text')
			.map((x) => (x.kind.c as MessagePaginateRespChunkKindText).content)
			.join('\n')
			.trim();
	}

	function getFileFromChunks(chunks: MessagePaginateRespChunk[]): { id: number; name: string }[] {
		return chunks
			.filter((x) => x.kind.t == 'file')
			.map((x) => x.kind.c as MessagePaginateRespChunkKindFile);
	}
</script>

{#each getMessages() as msg}
	{#key msg.id}
		{#if msg.role == Role.User}
			{@const content = getTextFromChunks(msg.chunks)}
			{@const files = getFileFromChunks(msg.chunks)}
			<User
				{content}
				{files}
				onupdate={(text) => {
					if (room == undefined) return;
					if (room.model_id == undefined) dispatchError('internal', 'select a model first');
					else
						mutate({
							chat_id: chatId,
							model_id: room.model_id,
							mode: room.mode,
							text,
							files,
							msgId: msg.id
						});
				}}
			/>
		{:else if msg.role == Role.Assistant}
			{@const streaming = msg.stream}
			<ResponseBox>
				<Chunks chunks={msg.chunks} monochrome={streaming} />
				<ResponseEdit content={getTextFromChunks(msg.chunks)} token={msg.token} cost={msg.price} />
			</ResponseBox>
		{/if}
	{/key}
{/each}
