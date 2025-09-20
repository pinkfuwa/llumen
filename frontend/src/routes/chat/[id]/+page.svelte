<script lang="ts">
	let { params } = $props();
	import { MessageInput } from '$lib/components';
	import MessagePagination from '$lib/components/message/MessagePagination.svelte';
	import Copyright from '$lib/components/Copyright.svelte';
	import { createMessage } from '$lib/api/message';
	import { _ } from 'svelte-i18n';
	import { MessageCreateReqMode as Mode } from '$lib/api/types';
	import { haltCompletion, useRoom, useRoomStreamingState } from '$lib/api/chatroom.js';

	let id = $derived(Number(params.id));

	let { mutate } = createMessage();
	let { mutate: halt } = haltCompletion();

	let content = $state('');
	let files = $state([]);
	let mode = $state(Mode.Normal);
	let title = $state<string | null>(null);

	let { data: room } = $derived(id == undefined ? useRoom(id) : { data: undefined });

	let isStreaming = $derived(useRoomStreamingState(id));
</script>

<svelte:head>
	{#if title != null}
		<title>{title}</title>
	{:else}
		<title>Chatroom {params.id}</title>
	{/if}
</svelte:head>

<Copyright top />

<main class="nobar flex h-full flex-col-reverse overflow-y-auto">
	<div class="sticky bottom-2 z-10 mt-4 flex justify-center">
		<MessageInput
			above
			selectionDisabled
			bind:content
			modelId={$room?.model_id}
			bind:mode
			bind:files
			onsubmit={() => {
				mutate({ chat_id: id, text: content, mode });
				content = '';
				isStreaming.set(true);
			}}
			oncancel={() => {
				halt({ id });
				isStreaming.set(false);
			}}
			disabled={$isStreaming}
		/>
	</div>
	{#key id}
		<MessagePagination {id} />
	{/key}
	<div class="min-h-16"></div>
</main>
