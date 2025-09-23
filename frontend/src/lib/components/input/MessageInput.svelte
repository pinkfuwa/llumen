<script lang="ts">
	import { createDropZone } from '@sv-use/core';
	import MdTextbox from './MDTextbox.svelte';
	import ModeBtn from './ModeBtn.svelte';
	import UploadBtn from './UploadBtn.svelte';
	import SendBtn from './SendBtn.svelte';
	import FileGroup from '../buttons/FileGroup.svelte';
	import ModelBtn from './ModelBtn.svelte';
	import MarkdownBtn from './MarkdownBtn.svelte';
	import { _ } from 'svelte-i18n';
	import StopBtn from './StopBtn.svelte';
	import { afterNavigate } from '$app/navigation';
	import { MessageCreateReqMode as Mode } from '$lib/api/types';

	let {
		mode = $bindable(Mode.Normal),
		files = $bindable([] as Array<File>),
		modelId = $bindable<string | undefined>(undefined),
		content = $bindable(''),
		onsubmit = undefined as undefined | (() => void),
		oncancel = undefined as undefined | (() => void),
		above = false,
		selectionDisabled = false,
		disabled = false
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

	afterNavigate(() => {
		content = '';
		editable = true;
	});
</script>

<div
	class="min-h-sm item relative mx-auto rounded-md border border-outline bg-chat-input-bg p-2 shadow-xl shadow-secondary md:w-120 lg:w-150 xl:w-200"
	bind:this={container}
>
	{#if dropZone.isOver && editable}
		<div
			class="absolute top-0 -left-0 flex h-full w-full items-center justify-center rounded-lg bg-primary text-2xl"
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
		<MdTextbox
			bind:editable
			placeholder={disabled ? $_('chat.stop_first') : $_('chat.question')}
			bind:value={content}
			{onsubmit}
			{disabled}
		/>
		{#if disabled}
			<StopBtn onclick={oncancel} />
		{:else}
			<SendBtn onclick={onsubmit} />
		{/if}
	</div>
	<div class="flex flex-row items-center justify-between">
		<div class="flex h-11 grow items-center space-x-2">
			<ModelBtn bind:value={modelId} {above} disabled={selectionDisabled} />
			<ModeBtn bind:value={mode} />
			<UploadBtn bind:files />
		</div>
		{#if content.length != 0}
			<MarkdownBtn bind:editable />
		{/if}
	</div>
</div>
