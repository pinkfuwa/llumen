<script lang="ts">
	import MdTextbox from '$lib/components/MDTextbox.svelte';
	import SearchBtn from '$lib/components/buttons/SearchBtn.svelte';
	import UploadBtn from '$lib/components/buttons/UploadBtn.svelte';
	import SendBtn from '$lib/components/buttons/SendBtn.svelte';
	import FileGroup from '$lib/components/FileGroup.svelte';
	import ModelBtn from '$lib/components/buttons/ModelBtn.svelte';
	import MarkdownBtn from '$lib/components/buttons/MarkdownBtn.svelte';
	import { createRoom } from '$lib/api/chatroom';
	import { goto } from '$app/navigation';

	let mode = $state(0) as 0 | 1 | 2;
	let editable = $state(true);
	let files = $state<File[]>([]);
	let modelId = $state('');
	let content = $state('');

	let createRoomMutation = createRoom();
</script>

<h1 class="mx-auto mb-8 text-4xl font-light lg:text-5xl">Ask anything</h1>
<div
	class="min-h-sm item mx-auto rounded-md border border-outline p-2 md:w-md lg:w-[calc(30vw+300px)] xl:w-[700px]"
>
	{#if files.length != 0}
		<div class="mb-2 overflow-scroll border-b border-outline pb-2">
			<FileGroup {files} />
		</div>
	{/if}
	<div class="mb-2 flex items-center justify-between space-x-2 border-b border-outline p-2 pb-4">
		<MdTextbox {editable} placeholder="Enter your question here" bind:value={content} />
		<SendBtn
			onclick={() => {
				$createRoomMutation.mutate(
					{
						firstMessage: content,
						modelId,
						files,
						mode
					},
					{
						onSuccess: (data) => {
							console.log('!', { data });
							goto('/chat/' + encodeURIComponent(data.id));
						}
					}
				);
			}}
		/>
	</div>
	<div class="flex flex-row items-center justify-between">
		<div class="flex grow items-center space-x-1">
			<ModelBtn bind:value={modelId} />
			<SearchBtn bind:value={mode} />
			<UploadBtn bind:files />
		</div>
		<MarkdownBtn bind:editable />
	</div>
</div>
