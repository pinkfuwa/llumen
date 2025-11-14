<script lang="ts">
	import { createDropZone } from './dropzone.svelte';
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
	import { ChatMode as Mode, type ModelListResp } from '$lib/api/types';
	import { getContext, setContext, untrack } from 'svelte';
	import { get, writable, type Readable } from 'svelte/store';
	import { FileUp } from '@lucide/svelte';
	import { dispatchError } from '$lib/error';

	let {
		mode = $bindable(Mode.Normal),
		files = $bindable([] as Array<File>),
		modelId = $bindable<string | undefined>(undefined),
		content = $bindable(''),
		onsubmit = undefined as undefined | (() => void),
		oncancel = undefined as undefined | (() => void),
		disabled = false
	} = $props();

	let editable = $state(true);

	let container = $state<HTMLElement | null>();

	const filetypes = writable('*');
	setContext('filetypes', filetypes);

	const dropZone = createDropZone(() => container, {
		allowedDataTypes: () => $filetypes,
		multiple: false,
		onDrop(newFiles: File[] | null) {
			if (newFiles == null) return;
			newFiles.forEach((f) => files.push(f));
		}
	});

	// FIXME: should clear state on upper layer with props
	afterNavigate((after) => {
		if (after.to?.route.id == '/chat/[id]') content = '';
		editable = true;
	});

	// reset mode when tool is not supported
	// FIXME: model isn't sync when updated, only sync on delete/create
	$effect(() => {
		if (mode == Mode.Normal) return;
		const models = get(getContext<Readable<ModelListResp | undefined>>('models'));
		if (models == undefined) return;
		const model = models.list.find((m) => m.id == modelId);
		if (model == undefined) return;
		if (model.tool) return;
		untrack(() => (mode = Mode.Normal));
		dispatchError('internal', "the model doesn't support tool");
	});

	function submit() {
		if (onsubmit && content.length > 0) {
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
			class="absolute top-0 -left-0 flex h-full w-full items-center justify-center rounded-lg bg-primary text-2xl"
		>
			<FileUp />
			Upload File
		</div>
	{/if}
	{#if files.length != 0}
		<div class="mb-2 overflow-auto border-b border-outline pb-2">
			<FileGroup {files} deletable />
		</div>
	{/if}
	<div
		class="mb-2 flex flex-row items-center justify-between space-x-2 border-b border-outline pr-2 pb-2"
	>
		<MdTextbox
			bind:editable
			placeholder={disabled ? $_('chat.stop_first') : $_('chat.question')}
			bind:value={content}
			onsubmit={submit}
			{disabled}
		/>
		{#if disabled}
			<StopBtn onclick={oncancel} />
		{:else}
			<SendBtn onclick={submit} />
		{/if}
	</div>
	<div class="flex flex-row items-center justify-between">
		<div class="flex h-11 w-full grow items-center justify-start space-x-2">
			<ModelBtn bind:value={modelId} />
			<ModeBtn bind:value={mode} />
			<UploadBtn bind:files />
		</div>
		{#if content.length != 0}
			<MarkdownBtn bind:editable />
		{/if}
	</div>
</div>
