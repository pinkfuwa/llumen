<script lang="ts">
	import { createRoom } from '$lib/api/chatroom';
	import { fade } from 'svelte/transition';
	import { MessageInput, Copyright } from '$lib/components';
	import { goto } from '$app/navigation';
	import { _ } from 'svelte-i18n';
	import { titleGrad } from '$lib/preference';

	let { mutate } = createRoom();

	let content = $state('');
	let modelId = $state<number | null>(null);
	let files = $state([]);
	let mode = $state(0 as 0);
</script>

<svelte:head>
	<title>{$_('chat.title')}</title>
</svelte:head>

<main class="flex h-full w-full flex-col justify-center">
	<h1
		class="mx-auto mb-4 bg-gradient-to-r {$titleGrad} bg-clip-text pb-4 text-4xl font-semibold text-transparent select-none md:text-5xl lg:text-6xl"
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
</main>

<Copyright />
