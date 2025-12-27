<script lang="ts">
	let { params } = $props();
	import { MessageInput } from '$lib/components';
	import MessagePagination from '$lib/components/message/MessagePagination.svelte';
	import Hallucination from '$lib/components/common/Hallucination.svelte';
	import { _ } from 'svelte-i18n';
	import { ChatMode as Mode } from '$lib/api/types';
	import { haltCompletion, useRoom } from '$lib/api/chatroom.svelte';
	import { createUploadEffect } from '$lib/api/files.svelte';
	import Scroll from '$lib/ui/Scroll.svelte';
	import { createMessage, getStream } from '$lib/api/message.svelte.js';
	import { untrack } from 'svelte';
	import { afterNavigate } from '$app/navigation';

	let id = $derived(Number(params.id));

	let { mutate } = createMessage();
	let { mutate: halt } = haltCompletion();

	let content = $state('');
	let files: File[] = $state([]);
	let mode = $state<Mode>(Mode.Normal);

	let { data: room } = $derived(useRoom(id));

	let modelId = $state<string | null>(null);

	let inited = false;

	afterNavigate(() => {
		inited = false;
	});

	$effect(() => {
		if ($room == undefined) return;
		untrack(() => {
			if (inited) return;
			inited = true;
			mode = $room.mode;
			if ($room?.model_id) modelId = $room?.model_id.toString();
		});
	});

	const ensureUploaded = createUploadEffect(() => files);

	// svelte 5 bug
	let stream = $state(false);
	getStream((x) => (stream = x));
</script>

<Hallucination />

<Scroll class="nobar flex h-full flex-col-reverse">
	<div class="sticky bottom-1 z-10 mt-4 flex justify-center">
		<MessageInput
			bind:content
			bind:modelId
			bind:mode
			bind:files
			onsubmit={async () => {
				const uploadedFiles = await ensureUploaded();
				mutate({
					chat_id: id,
					text: content,
					mode: mode!,
					model_id: parseInt(modelId!),
					files: uploadedFiles
				});
				content = '';
				files = [];
			}}
			oncancel={() => {
				halt({ id });
			}}
			disabled={stream || modelId === null || mode === null}
		/>
	</div>
	<MessagePagination room={$room} />
	<div class="min-h-16"></div>
</Scroll>
