<script lang="ts">
	let { params } = $props();
	import { MessageInput } from '$lib/components';
	import MessagePagination from '$lib/components/message/MessagePagination.svelte';
	import Copyright from '$lib/components/Copyright.svelte';
	import { createMessage } from '$lib/api/message';
	import { _ } from 'svelte-i18n';
	import { ChatMode as Mode } from '$lib/api/types';
	import { haltCompletion, useRoom, useRoomStreamingState } from '$lib/api/chatroom.js';
	import { UploadManager } from '$lib/api/files.js';

	let id = $derived(Number(params.id));

	let { mutate } = createMessage();
	let { mutate: halt } = haltCompletion();

	let modelId = $state<null | number>(null);
	let content = $state('');
	let files: File[] = $state([]);
	let mode = $state<Mode | null>(null);

	let { data: room } = $derived(useRoom(id));

	$effect(() => {
		if ($room == undefined) return;
		if (modelId == null && $room?.model_id) modelId = $room?.model_id;
		if (mode == null) mode = $room.mode;
	});

	let isStreaming = $derived(useRoomStreamingState(id));

	let uploadManager = $derived(new UploadManager(id));

	$effect(() => {
		uploadManager.retain(files);
	});
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
			onsubmit={async () => {
				isStreaming.set(true);
				mutate({
					chat_id: id,
					text: content,
					mode: mode!,
					model_id: modelId!,
					files: await uploadManager.getUploads(files)
				});
				content = '';
				files = [];
			}}
			oncancel={() => {
				halt({ id });
				isStreaming.set(false);
			}}
			disabled={$isStreaming || modelId === null || mode === null}
		/>
	</div>
	{#key id}
		<MessagePagination {id} room={$room} />
	{/key}
	<div class="min-h-16"></div>
</main>
