<script lang="ts">
	import type { PageEntry } from '$lib/api/state';
	import {
		MessagePaginateRespRole as Role,
		type MessagePaginateRespChunk,
		type MessagePaginateRespChunkKindFile,
		type MessagePaginateRespChunkKindText,
		type MessagePaginateRespList
	} from '$lib/api/types';
	import ResponseBox from './buttons/ResponseBox.svelte';
	import ResponseEdit from './buttons/ResponseEdit.svelte';
	import User from './buttons/User.svelte';
	import Chunks from './Chunks.svelte';

	let div = $state<HTMLElement | null>(null);

	const { entry }: { entry: PageEntry<MessagePaginateRespList> } = $props();
	const data = entry.data;

	$effect(() => entry.target.set(div));

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

<div class="mt-2 flex flex-col-reverse space-y-2" bind:this={div}>
	{#each $data as msg}
		{#if msg.role == Role.User}
			{@const content = getTextFromChunks(msg.chunks)}
			{@const files = getFileFromChunks(msg.chunks)}
			<User {content} {files} />
		{:else if msg.role == Role.Assistant}
			<ResponseBox>
				<Chunks chunks={msg.chunks} />
				<ResponseEdit content={getTextFromChunks(msg.chunks)} token={msg.token} cost={msg.price} />
			</ResponseBox>
		{/if}
	{:else}
		<div class="h-1"></div>
	{/each}
</div>
