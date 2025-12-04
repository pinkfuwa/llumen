<script lang="ts">
	import { createDropZone } from './dropzone.svelte';
	import Textbox from './Textbox.svelte';
	import ModeSelector from './ModeSelector.svelte';
	import UploadBtn from './Upload.svelte';
	import Send from './Send.svelte';
	import FileGroup from '../common/FileGroup.svelte';
	import ModelSelector from './ModelSelector.svelte';
	import MarkdownBtn from './MarkdownBtn.svelte';
	import { _ } from 'svelte-i18n';
	import Stop from './Stop.svelte';
	import { afterNavigate } from '$app/navigation';
	import { ChatMode as Mode, type ModelListResp } from '$lib/api/types';
	import { getContext } from 'svelte';
	import { type Readable } from 'svelte/store';
	import { FileUp } from '@lucide/svelte';
	import { getSupportedFileTypes } from './fileTypes';

	let {
		mode = $bindable(Mode.Normal),
		files = $bindable([] as Array<File>),
		modelId = $bindable<string | null>(null),
		content = $bindable(''),
		onsubmit = undefined as undefined | (() => void),
		oncancel = undefined as undefined | (() => void),
		disabled = false
	}: {
		mode?: Mode;
		files?: Array<File>;
		modelId: string | null;
		content?: string;
		onsubmit?: () => void;
		oncancel?: () => void;
		disabled?: boolean;
	} = $props();

	let editable = $state(true);

	let container = $state<HTMLElement | null>();

	const models = getContext<Readable<ModelListResp | undefined>>('models');

	const modelIdValid = $derived(
		modelId != null &&
			($models == undefined || $models.list.some((m) => m.id.toString() == modelId))
	);

	let selectModelCap = $derived.by(() => {
		let uselessFn = (a: any) => {};
		uselessFn(modelId);
		return $models?.list.find((x) => x.id.toString() == modelId);
	});
	let filetypes = $derived(
		selectModelCap == undefined ? '*' : getSupportedFileTypes(selectModelCap)
	);

	const dropZone = createDropZone(() => container, {
		allowedDataTypes: () => filetypes,
		multiple: false,
		onDrop(newFiles: File[] | null) {
			if (newFiles == null) return;
			newFiles.forEach((f) => files.push(f));
		}
	});

	afterNavigate((after) => {
		content = '';
		editable = true;
		modelId = null;
	});

	function submit() {
		if (onsubmit && content.length > 0 && modelIdValid) {
			disabled = true;
			onsubmit();
		}
	}
</script>

<div
	class="min-h-sm item relative mx-auto w-[90%] rounded-md border border-outline bg-chat-input-bg p-2 shadow-xl shadow-secondary md:w-[min(750px,75%)]"
	bind:this={container}
	onpaste={(event) => {
		const clipboardData = event.clipboardData;
		if (clipboardData == null || clipboardData.files.length == 0) return;
		for (let i = 0; i < clipboardData.files.length; i++) {
			files.push(clipboardData.files[i]);
		}
	}}
>
	{#if dropZone.isOver && editable}
		<div
			class="absolute top-0 left-0 flex h-full w-full items-center justify-center rounded-lg bg-primary text-2xl"
		>
			<FileUp />
			Upload File
		</div>
	{/if}
	{#if files.length != 0}
		<div class="mb-2 max-h-[60vh] overflow-y-auto border-b border-outline pb-2">
			<FileGroup {files} deletable />
		</div>
	{/if}
	<div
		class="mb-2 flex flex-row items-center justify-between space-x-2 border-b border-outline pr-2 pb-2"
	>
		<Textbox
			bind:editable
			placeholder={disabled ? $_('chat.stop_first') : $_('chat.question')}
			bind:value={content}
			onsubmit={submit}
			{disabled}
		/>
		{#if disabled}
			<Stop onclick={oncancel} />
		{:else}
			<Send onclick={submit} disabled={content.length == 0 || !modelIdValid} />
		{/if}
	</div>
	<div class="flex flex-row items-center justify-between">
		<div class="flex h-11 w-full grow items-center justify-start space-x-2">
			<ModelSelector bind:value={modelId} />
			<ModeSelector bind:value={mode} limited={!selectModelCap?.tool} />
			<UploadBtn bind:files {filetypes} />
		</div>
		{#if content.length != 0}
			<MarkdownBtn bind:editable />
		{/if}
	</div>
</div>
