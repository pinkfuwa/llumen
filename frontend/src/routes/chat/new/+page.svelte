<script lang="ts">
	import { createRoom } from '$lib/api/chatroom';
	import { _ } from 'svelte-i18n';
	import { fade } from 'svelte/transition';
	import MessageInput from '$lib/components/MessageInput.svelte';
	import { goto } from '$app/navigation';

	let { mutate } = createRoom();

	let content = $state('');
	let modelId = $state<number | null>(null);
	let files = $state([]);
	let mode = $state(0 as 0);
</script>

<div class="flex h-full w-full flex-col justify-center">
	<h1
		class="mx-auto mb-4 bg-gradient-to-r from-dark to-blue-600 bg-clip-text pb-4 text-4xl font-semibold text-transparent md:text-5xl lg:text-6xl"
		in:fade={{ duration: 150 }}
	>
		{$_('chat.welcome')}
	</h1>
	<MessageInput
		bind:content
		bind:modelId
		bind:mode
		bind:files
		initSelect
		onsubmit={() => {
			if (modelId == null) return;

			mutate(
				{
					message: content,
					modelId,
					files,
					mode
				},
				(data) => {
					goto('/chat/' + encodeURIComponent(data.id));
				}
			);
		}}
	/>
</div>
