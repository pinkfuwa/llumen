<script lang="ts">
	import { createDropZone } from '@sv-use/core';
	import MdTextbox from '$lib/components/MDTextbox.svelte';
	import SearchBtn from '$lib/components/buttons/SearchBtn.svelte';
	import UploadBtn from '$lib/components/buttons/UploadBtn.svelte';
	import SendBtn from '$lib/components/buttons/SendBtn.svelte';
	import FileGroup from '$lib/components/FileGroup.svelte';
	import ModelBtn from '$lib/components/buttons/ModelBtn.svelte';
	import MarkdownBtn from '$lib/components/buttons/MarkdownBtn.svelte';
	import { createRoom } from '$lib/api/chatroom';
	import { goto } from '$app/navigation';
	import { _ } from 'svelte-i18n';
	import { fade } from 'svelte/transition';

	let mode = $state(0) as 0 | 1 | 2;
	let editable = $state(true);
	let files = $state<File[]>([]);
	let modelId = $state('');
	let content = $state('');

	let { mutate } = createRoom();

	let container = $state<HTMLElement | null>();

	const dropZone = createDropZone(() => container, {
		allowedDataTypes: '*',
		multiple: false,
		onDrop(files: File[] | null) {
			if (files != null) {
				files.forEach((f) => files.push(f));
			}
		}
	});
</script>

<svelte:head>
	<title>{$_('chat.title')}</title>
</svelte:head>

<h1
	class="mx-auto mb-4 bg-gradient-to-r from-dark to-blue-600 bg-clip-text pb-4 text-4xl font-semibold text-transparent lg:text-5xl"
	in:fade={{ duration: 150 }}
>
	{$_('chat.welcome')}
</h1>
<div
	class="min-h-sm item relative mx-auto rounded-md border border-outline p-2 shadow-xl shadow-hover md:w-md lg:w-[calc(30vw+300px)] xl:w-[700px]"
	bind:this={container}
>
	{#if dropZone.isOver && editable}
		<div
			class="absolute top-0 -left-0 flex h-full w-full items-center justify-center rounded-lg bg-light text-2xl"
		>
			Upload File
		</div>
	{/if}
	{#if files.length != 0}
		<div class="mb-2 overflow-scroll border-b border-outline pb-2">
			<FileGroup {files} />
		</div>
	{/if}
	<div class="mb-2 flex items-center justify-between space-x-2 border-b border-outline p-2 pb-4">
		<MdTextbox {editable} placeholder={$_('chat.question')} bind:value={content} />
		<SendBtn
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
	</div>
	<div class="flex flex-row items-center justify-between">
		<div class="flex grow items-center space-x-1">
			<ModelBtn bind:value={modelId} />
			<SearchBtn bind:value={mode} />
			<UploadBtn bind:files />
		</div>
		{#if content.length != 0}
			<MarkdownBtn bind:editable />
		{/if}
	</div>
</div>
