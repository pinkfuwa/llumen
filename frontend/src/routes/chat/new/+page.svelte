<script lang="ts">
	import { createRoom } from '$lib/api/chatroom';
	import { _ } from 'svelte-i18n';
	import { fade } from 'svelte/transition';
	import MessageInput from '$lib/components/MessageInput.svelte';
	import { goto } from '$app/navigation';

	let { mutate } = createRoom();

	let content = $state('');
	let modelId = $state('');
	let files = $state([]);
	let mode = $state(0 as 0);
</script>

<h1
	class="mx-auto mb-4 bg-gradient-to-r from-dark to-blue-600 bg-clip-text pb-4 text-4xl font-semibold text-transparent lg:text-5xl"
	in:fade={{ duration: 150 }}
>
	{$_('chat.welcome')}
</h1>
<MessageInput
	bind:content
	bind:modelId
	bind:mode
	bind:files
	onclick={() => {
		mutate(
			{
				firstMessage: content,
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
