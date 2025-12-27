<script lang="ts">
	import { createRoom } from '$lib/api/chatroom.svelte';
	import { fade } from 'svelte/transition';
	import { MessageInput, Copyright } from '$lib/components';
	import { goto } from '$app/navigation';
	import { _ } from 'svelte-i18n';
	import { ChatMode as Mode } from '$lib/api/types';
	import { page } from '$app/state';
	import { PersistedState } from 'runed';
	import { createUploadEffect } from '$lib/api';

	let { mutate } = createRoom();

	let content = $state('');
	const modelId = new PersistedState<null | string>('DefaultModelId', null);
	let files = $state([]);
	let mode = $state(Mode.Normal);

	const ensureUploaded = createUploadEffect(() => files);

	$effect(() => {
		const param = page.url.searchParams;
		if (param.has('q')) content = param.get('q')!;
	});
</script>

<svelte:head>
	<title>{$_('chat.title')}</title>
</svelte:head>

<main class="flex h-full w-full flex-col justify-center">
	<h1
		class="mx-auto mb-4 bg-gradient-to-r from-secondary to-primary bg-clip-text pb-4 text-[11vw] font-semibold text-transparent select-none md:text-[max(4rem,5vw)]"
		in:fade={{ duration: 150 }}
	>
		{$_('chat.welcome')}
	</h1>
	<MessageInput
		bind:content
		bind:modelId={modelId.current}
		bind:mode
		bind:files
		onsubmit={async () => {
			if (modelId.current == null) return;

			const uploadedFiles = await ensureUploaded();

			mutate(
				{
					message: content,
					modelId: parseInt(modelId.current),
					files: uploadedFiles,
					mode
				},
				(data) => goto('/chat/' + encodeURIComponent(data.id))
			);
		}}
		large
	/>
</main>

<Copyright />
