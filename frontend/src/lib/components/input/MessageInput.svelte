<script lang="ts">
	import Textbox from './Textbox.svelte';
	import ActionMenu from './ActionMenu.svelte';
	import Send from './Send.svelte';
	import Stop from './Stop.svelte';
	import ModelSelector from './ModelSelector.svelte';
	import MarkdownBtn from './MarkdownBtn.svelte';
	import UnsupportedFilesModal from './UnsupportedFilesModal.svelte';
	import FileGroup from '../common/FileGroup.svelte';
	import { createDropZone } from './dropzone.svelte';
	import { afterNavigate } from '$app/navigation';
	import { streaming } from '$lib/api/message.svelte';
	import { FileUp } from '@lucide/svelte';
	import { t } from 'svelte-intl-precompile';
	import { inputContent, inputFiles, addFiles, submit, effective, isEditing } from './state.svelte';

	let {
		large = false
	}: {
		large?: boolean;
	} = $props();

	let container = $state<HTMLElement | null>(null);

	let dropZone = $state(
		createDropZone(() => container, {
			onDrop(newFiles: File[] | null) {
				if (newFiles == null) return;
				addFiles(newFiles);
			},
			onPaste(newFiles: File[] | null) {
				if (newFiles == null) return;
				addFiles(newFiles);
			}
		})
	);

	afterNavigate(() => {
		isEditing.val = true;
	});
</script>

<UnsupportedFilesModal />
<div
	role="region"
	class="min-h-sm item shadow-accent-soft relative mx-auto w-[90%] space-y-2 rounded-md border border-border bg-card p-2 shadow-xl md:w-[min(750px,75%)]"
	bind:this={container}
>
	{#if dropZone.isOver && isEditing.val}
		<div
			class="absolute top-0 left-0 flex h-full w-full items-center justify-center rounded-md border-2 border-dashed border-border bg-muted/40 text-2xl text-primary"
		>
			<FileUp />
			{$t('chat.upload_file')}
		</div>
	{/if}
	{#if inputFiles.val.length != 0}
		<div class="mb-2 max-h-[60vh] overflow-y-auto border-b border-border pb-2">
			<FileGroup files={inputFiles.val} mimes={effective.supportedMimes} deletable />
		</div>
	{/if}
	<div class="flex flex-row items-center justify-between space-x-2 pr-2">
		<Textbox minRow={large ? 2 : 1} />
		{#if streaming.val}
			<Stop />
		{:else}
			<Send onclick={submit} />
		{/if}
	</div>
	<div class="flex flex-row items-center justify-between">
		<div class="flex h-11 w-full grow items-center justify-start space-x-2">
			<ModelSelector />
			<ActionMenu />
		</div>
		{#if inputContent.val.length != 0}
			<MarkdownBtn />
		{/if}
	</div>
</div>
