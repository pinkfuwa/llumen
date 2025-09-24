<script lang="ts">
	let { params } = $props();
	import { MessageInput } from '$lib/components';
	import MessagePagination from '$lib/components/message/MessagePagination.svelte';
	import Copyright from '$lib/components/Copyright.svelte';
	import { createMessage } from '$lib/api/message';
	import { _ } from 'svelte-i18n';
	import { ChatMode as Mode } from '$lib/api/types';
	import { haltCompletion, useRoom, useRoomStreamingState } from '$lib/api/chatroom.js';

	let id = $derived(Number(params.id));

	let { mutate } = createMessage();
	let { mutate: halt } = haltCompletion();

	let modelId = $state<null | number>(null);
	let content = $state('');
	let files: File[] = $state([]);
	let mode = $state<Mode | null>(null);
	let title = $state<string | null>(null);

	let { data: room } = $derived(useRoom(id));

	$effect(() => {
		if ($room == undefined) return;
		if (modelId == null && $room?.model_id) modelId = $room?.model_id;
		if (mode == null) mode = $room.mode;
	});

	let isStreaming = $derived(useRoomStreamingState(id));
</script>

<Copyright top />

<main class="nobar flex h-full flex-col-reverse overflow-y-auto">
	<div class="sticky bottom-2 z-10 mt-4 flex justify-center">
		<MessageInput
			above
			bind:content
			modelId={$room?.model_id}
			bind:mode
			bind:files
			onsubmit={() => {
				mutate({ chat_id: id, text: content, mode: mode!, model_id: modelId!, files: [] });
				content = '';
				isStreaming.set(true);
			}}
			oncancel={() => {
				halt({ id });
				isStreaming.set(false);
			}}
			disabled={$isStreaming || modelId === null || mode === null}
		/>
	</div>
	{#key id}
		<MessagePagination {id} />
	{/key}
	<div class="min-h-16"></div>
</main>
