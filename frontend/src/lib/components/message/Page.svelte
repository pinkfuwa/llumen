<script lang="ts">
	import type { PageEntry } from '$lib/api/state';
	import {
		MessagePaginateRespRole as Role,
		type MessagePaginateRespChunk,
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

	function getRespFromChunks(chunks: MessagePaginateRespChunk[]) {
		return chunks
			.filter((x) => x.kind.t == 'text')
			.map((x) => x.kind.c)
			.join('\n');
	}
</script>

<div class="mt-2 flex flex-col-reverse space-y-2" bind:this={div}>
	{#each $data as msg}
		{#if msg.role == Role.User}
			<User content={(msg.chunks[0].kind.c as MessagePaginateRespChunkKindText).context} />
		{:else if msg.role == Role.Assistant}
			<ResponseBox>
				<Chunks chunks={msg.chunks} />
				<ResponseEdit content={getRespFromChunks(msg.chunks)} />
			</ResponseBox>
		{/if}
	{:else}
		<div class="h-1"></div>
	{/each}
</div>
