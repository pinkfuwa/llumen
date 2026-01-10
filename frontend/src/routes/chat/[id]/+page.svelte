<script lang="ts">
	let { params } = $props();
	import { MessageInput } from '$lib/components';
	import MessagePagination from '$lib/components/message/MessagePagination.svelte';
	import Hallucination from '$lib/components/common/Hallucination.svelte';
	import { _ } from 'svelte-i18n';
	import { ChatMode as Mode } from '$lib/api/types';
	import { haltCompletion, useRoomQueryEffect, getCurrentRoom, getMessages } from '$lib/api';
	import { createUploadEffect } from '$lib/api/files.svelte';
	import Scroll from '$lib/ui/Scroll.svelte';
	import { createMessage, getStream } from '$lib/api';
	import { untrack } from 'svelte';
	import { afterNavigate } from '$app/navigation';

	let { mutate } = createMessage();
	let { mutate: halt } = haltCompletion();

	let content = $state('');
	let files: File[] = $state([]);
	let mode = $state<Mode>(Mode.Normal);

	let id = $derived(Number(params.id));

	$effect(() => {
		useRoomQueryEffect(id);
	});

	let room = $derived(getCurrentRoom());

	let modelId = $state<string | null>(null);

	let inited = false;

	afterNavigate(() => {
		inited = false;
	});

	$effect(() => {
		if (room == undefined) return;
		untrack(() => {
			if (inited) return;
			inited = true;
			mode = room.mode;
			if (room?.model_id) modelId = room?.model_id.toString();
		});
	});

	const ensureUploaded = createUploadEffect(() => files);

	let stream = $derived(getStream().stream);
</script>

<Hallucination />

<Scroll class="nobar flex h-full flex-col-reverse" key={getMessages().messages.length}>
	<div class="sticky bottom-1 z-10 mt-4 flex justify-center">
		<MessageInput
			bind:content
			bind:modelId
			bind:mode
			bind:files
			onsubmit={async () => {
				const uploadedFiles = await ensureUploaded();
				const currentId = id;
				mutate({
					chat_id: currentId,
					text: content,
					mode: mode!,
					model_id: parseInt(modelId!),
					files: uploadedFiles
				});
				content = '';
				files = [];
			}}
			oncancel={() => {
				const currentId = id;
				halt({ id: currentId });
			}}
			disabled={stream || modelId === null || mode === null}
		/>
	</div>
	<MessagePagination {room} />
	<div class="min-h-16"></div>
</Scroll>
