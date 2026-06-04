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
	import { separateFiles } from './fileTypes';
	import { afterNavigate, beforeNavigate } from '$app/navigation';
	import { streaming } from '$lib/api/message.svelte';
	import { FileUp } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import {
		inputContent,
		inputFiles,
		submitting,
		displayModelId,
		displayMode,
		supportedMimes,
		submit,
		abortStream,
		onModelChange,
		onModeChange
	} from './state.svelte';

	const inputAreaStyle =
		'min-h-sm item relative mx-auto w-[90%] space-y-2 rounded-md border border-border bg-card p-2 shadow-xl shadow-accent-soft md:w-[min(750px,75%)]';

	let {
		large = false
	}: {
		large?: boolean;
	} = $props();

	let container = $state<HTMLElement | null>(null);
	let isEditing = $state(true);
	let showUnsupportedModal = $state(false);
	let pendingFiles: File[] = $state([]);
	let pendingUnsupportedFiles: File[] = $state([]);

	let dropZone = $state(
		createDropZone(() => container, {
			onDrop(newFiles: File[] | null) {
				if (newFiles == null) return;
				handleNewFiles(newFiles);
			},
			onPaste(newFiles: File[] | null) {
				if (newFiles == null) return;
				handleNewFiles(newFiles);
			}
		})
	);

	function handleNewFiles(newFiles: File[]) {
		const { supported, unsupported } = separateFiles(newFiles, supportedMimes.val);

		if (unsupported.length > 0) {
			pendingFiles = supported;
			pendingUnsupportedFiles = unsupported;
			showUnsupportedModal = true;
		} else {
			for (const f of supported) inputFiles.val.push(f);
		}
	}

	function uploadAllFiles() {
		for (const f of [...pendingFiles, ...pendingUnsupportedFiles]) inputFiles.val.push(f);
		showUnsupportedModal = false;
		pendingFiles = [];
		pendingUnsupportedFiles = [];
	}

	function uploadSupportedOnly() {
		for (const f of pendingFiles) inputFiles.val.push(f);
		showUnsupportedModal = false;
		pendingFiles = [];
		pendingUnsupportedFiles = [];
	}

	afterNavigate(() => {
		isEditing = true;
	});

	beforeNavigate(() => {});
</script>

<UnsupportedFilesModal
	bind:open={showUnsupportedModal}
	unsupportedFiles={pendingUnsupportedFiles}
	onUploadAll={uploadAllFiles}
	onUploadSupported={uploadSupportedOnly}
/>

<div role="region" class={inputAreaStyle} bind:this={container}>
	{#if dropZone.isOver && isEditing}
		<div
			class="absolute top-0 left-0 flex h-full w-full items-center justify-center rounded-md border-2 border-dashed border-border bg-muted/40 text-2xl text-primary"
		>
			<FileUp />
			{m['chat.upload_file']()}
		</div>
	{/if}
	{#if inputFiles.val.length != 0}
		<div class="mb-2 max-h-[60vh] overflow-y-auto border-b border-border pb-2">
			<FileGroup files={inputFiles.val} mimes={supportedMimes.val} deletable />
		</div>
	{/if}
	<div class="flex flex-row items-center justify-between space-x-2 pr-2">
		<Textbox
			bind:isEditing
			placeholder={streaming.val ? m['chat.stop_first']() : m['chat.question']()}
			bind:value={inputContent.val}
			onsubmit={submit}
			disabled={streaming.val}
			minRow={large ? 2 : 1}
		/>
		{#if streaming.val}
			<Stop onclick={abortStream} />
		{:else}
			<Send
				onclick={submit}
				disabled={(inputContent.val.length == 0 && inputFiles.val.length == 0) ||
					displayModelId.val == null ||
					submitting.val}
			/>
		{/if}
	</div>
	<div class="flex flex-row items-center justify-between">
		<div class="flex h-11 w-full grow items-center justify-start space-x-2">
			<ModelSelector value={displayModelId.val} onchange={onModelChange} />
			<ActionMenu value={displayMode.val} onmodechange={onModeChange} />
		</div>
		{#if inputContent.val.length != 0}
			<MarkdownBtn bind:isEditing />
		{/if}
	</div>
</div>
