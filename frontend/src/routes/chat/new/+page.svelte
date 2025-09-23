<script lang="ts">
	import { createRoom } from '$lib/api/chatroom';
	import { fade } from 'svelte/transition';
	import { MessageInput, Copyright } from '$lib/components';
	import { goto } from '$app/navigation';
	import { _ } from 'svelte-i18n';
	import { ChatMode as Mode } from '$lib/api/types';

	let { mutate } = createRoom();

	let content = $state('');
	let modelId = $state<string | undefined>(undefined);
	let files = $state([]);
	let mode = $state(Mode.Normal);
</script>

<svelte:head>
	<title>{$_('chat.title')}</title>
</svelte:head>

<main class="flex h-full w-full flex-col justify-center">
	<h1
		class="mx-auto mb-4 bg-gradient-to-r from-secondary to-primary bg-clip-text pb-4 text-4xl font-semibold text-transparent select-none md:text-5xl lg:text-6xl"
		in:fade={{ duration: 150 }}
	>
		{$_('chat.welcome')}
	</h1>
	<MessageInput
		bind:content
		bind:modelId
		bind:mode
		bind:files
		onsubmit={() => {
			if (modelId == null) return;

			mutate(
				{
					message: content,
					modelId: parseInt(modelId),
					files,
					mode
				},
				(data) => {
					goto('/chat/' + encodeURIComponent(data.id));
				}
			);
		}}
	/>
</main>

<Copyright />
