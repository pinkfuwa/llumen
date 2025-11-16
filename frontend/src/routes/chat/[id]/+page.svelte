<script lang="ts">
	let { params } = $props();
	import { MessageInput } from '$lib/components';
	import MessagePagination from '$lib/components/message/MessagePagination.svelte';
	import Copyright from '$lib/components/Copyright.svelte';
	import { _ } from 'svelte-i18n';
	import { ChatMode as Mode } from '$lib/api/types';
	import { haltCompletion, useRoom } from '$lib/api/chatroom.svelte';
	import { UploadManager } from '$lib/api/files.js';
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
			if ($room?.model_id) modelId = $room?.model_id.toString();
			if (mode == null) mode = $room.mode;
		});
	});

	let uploadManager = $derived(new UploadManager(id));
	$effect(() => uploadManager.retain(files));

	// svelte 5 bug
	let stream = $state(false);
	getStream((x) => (stream = x));
</script>

<Copyright top />

<Scroll class="nobar flex h-full flex-col-reverse">
	<div class="sticky bottom-2 z-10 mt-4 flex justify-center">
		<MessageInput
			bind:content
			bind:modelId
			bind:mode
			bind:files
			onsubmit={async () => {
				mutate({
					chat_id: id,
					text: content,
					mode: mode!,
					model_id: parseInt(modelId!),
					files: await uploadManager.getUploads(files)
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
