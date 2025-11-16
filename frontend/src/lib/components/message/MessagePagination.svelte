<script lang="ts">
	import { getMessages, useSSEEffect, updateMessage } from '$lib/api/message.svelte';
	import { type ChatReadResp, type FileMetadata } from '$lib/api/types';
	import { dispatchError } from '$lib/error';
	import ResponseBox from './buttons/ResponseBox.svelte';
	import ResponseEdit from './buttons/ResponseEdit.svelte';
	import User from './buttons/User.svelte';
	import Chunks from './Chunks.svelte';
	import { page } from '$app/state';

	const { room }: { room: ChatReadResp | undefined } = $props();

	// FIXME: only use when if is presented
	const chatId = $derived(parseInt(page.params.id!));

	let { mutate } = updateMessage();

	useSSEEffect(() => chatId);
</script>

{#each getMessages() as msg}
	{#key msg.id}
		{@const streaming = msg.stream}
		{#if msg.inner.t == 'user'}
			{@const content = msg.inner.c.text}
			{@const files = msg.inner.c.files}
			<User
				{content}
				{files}
				{streaming}
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
		{:else if msg.inner.t == 'assistant'}
			{@const chunks = msg.inner.c}
			<ResponseBox>
				<Chunks {chunks} {streaming} />

				{#if streaming}
					<div class="space-y-4">
						<hr class="mx-3 animate-pulse rounded-md border-primary bg-primary p-1" />
						<hr class="mx-3 animate-pulse rounded-md border-primary bg-primary p-1" />
					</div>
				{:else}
					{@const text = chunks
						.filter((x) => x.t == 'text')
						.map((x) => x.c)
						.join('\n')
						.trim()}
					<ResponseEdit content={text} token={msg.token_count} cost={msg.price} />
				{/if}
			</ResponseBox>
		{/if}
	{/key}
{/each}
