<script lang="ts">
	let { params } = $props();
	import { MessageInput } from '$lib/components';
	import MessagePagination from '$lib/components/message/MessagePagination.svelte';
	import Copyright from '$lib/components/Copyright.svelte';
	import { createMessage } from '$lib/api/message';
	import { _ } from 'svelte-i18n';

	let id = $derived(Number(params.id));

	let { mutate } = createMessage();

	let content = $state('');
	let modelId = $state<number | null>(null);
	let files = $state([]);
	let mode = $state(0 as 0);

	let isStreaming = $state(false);
</script>

<svelte:head>
	<title>Chatroom {params.id}</title>
</svelte:head>

<Copyright top />

<div class="nobar flex h-full flex-col-reverse overflow-y-auto">
	<div class="sticky bottom-2 z-10 mt-4 flex justify-center">
		<MessageInput
			above
			selectionDisabled
			bind:content
			{modelId}
			{mode}
			bind:files
			onsubmit={() => {
				mutate({ chat_id: id, text: content });
				content = '';
			}}
			oncancel={() => {
				// TODO: cancel streaming
			}}
			disabled={isStreaming}
		/>
	</div>
	{#key id}
		<MessagePagination {id} bind:isStreaming />
	{/key}
	<div class="min-h-16"></div>
</div>
