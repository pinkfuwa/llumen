<script lang="ts">
	let { params } = $props();
	import { _ } from 'svelte-i18n';
	import MessageInput from '$lib/components/MessageInput.svelte';
	import MessagePagination from '$lib/components/message/MessagePagination.svelte';
	import { createMessage } from '$lib/api/message.js';

	let id = $derived(Number(params.id));

	let { mutate } = createMessage();

	let content = $state('');
	let modelId = $state<number | null>(null);
	let files = $state([]);
	let mode = $state(0 as 0);
</script>

<div class="nobar flex h-full flex-col-reverse overflow-y-auto">
	<div class="sticky bottom-2 z-10 mt-4 flex justify-center">
		<MessageInput
			above
			bind:content
			{modelId}
			{mode}
			bind:files
			onsubmit={() => {
				mutate({ chat_id: id, text: content });
				content = '';
			}}
		/>
	</div>
	{#key id}
		<MessagePagination {id} />
	{/key}
</div>
