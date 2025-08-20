<script lang="ts">
	import { createDropZone } from '@sv-use/core';
	import MdTextbox from './MDTextbox.svelte';
	import SearchBtn from './SearchBtn.svelte';
	import UploadBtn from './UploadBtn.svelte';
	import SendBtn from './SendBtn.svelte';
	import FileGroup from '../buttons/FileGroup.svelte';
	import ModelBtn from './ModelBtn.svelte';
	import MarkdownBtn from './MarkdownBtn.svelte';
	import { _ } from 'svelte-i18n';
	import type { MouseEventHandler } from 'svelte/elements';

	let {
		mode = $bindable(0 as 0 | 1 | 2),
		files = $bindable([] as Array<File>),
		modelId = $bindable<number | null>(null),
		content = $bindable(''),
		onsubmit = undefined as undefined | (() => void),
		above = false,
		initSelect = false
	} = $props();

	let editable = $state(true);

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

<div
	class="min-h-sm item relative mx-auto rounded-md border border-outline bg-light p-2 shadow-xl shadow-hover md:w-md lg:w-[calc(30vw+300px)] xl:w-[700px]"
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
			<FileGroup {files} deletable />
		</div>
	{/if}
	<div class="mb-2 flex items-center justify-between space-x-2 border-b border-outline pr-2 pb-2">
		<MdTextbox bind:editable placeholder={$_('chat.question')} bind:value={content} {onsubmit} />
		<SendBtn onclick={onsubmit} />
	</div>
	<div class="flex flex-row items-center justify-between">
		<div class="flex grow items-center space-x-1">
			<ModelBtn bind:value={modelId} {above} disabled={!initSelect} />
			<SearchBtn bind:value={mode} disabled={!initSelect} />
			<UploadBtn bind:files />
		</div>
		{#if content.length != 0}
			<MarkdownBtn bind:editable />
		{/if}
	</div>
</div>
