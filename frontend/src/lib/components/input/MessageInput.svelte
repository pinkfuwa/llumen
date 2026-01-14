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
	import { afterNavigate, beforeNavigate, goto } from '$app/navigation';
	import { ChatMode as Mode } from '$lib/api/types';
	import { getModels } from '$lib/api/model.svelte.js';
	import { FileUp } from '@lucide/svelte';
	import { getSupportedFileExtensions } from './fileTypes';
	import Confirm from './Confirm.svelte';
	import Modal from '$lib/ui/Modal.svelte';

	let {
		mode = $bindable(Mode.Normal),
		files = $bindable([]),
		modelId = $bindable(null),
		content = $bindable(''),
		onsubmit,
		oncancel,
		disabled = false,
		large = false
	}: {
		mode?: Mode;
		files?: Array<File>;
		modelId: string | null;
		content?: string;
		onsubmit?: () => void;
		oncancel?: () => void;
		disabled?: boolean;
		large?: boolean;
	} = $props();

	let editable = $state(true);
	let showConfirm = $state(false);
	let pendingNavigationUrl: string | null = $state(null);
	let showConvertToFileDialog = $state(false);
	let convertFileName = $state('');

	let container = $state<HTMLElement | null>();

	const models = $derived(getModels());

	const modelIdValid = $derived(
		modelId != null && (models == undefined || models.list.some((m) => m.id.toString() == modelId))
	);

	let selectModelCap = $derived.by(() => {
		let uselessFn = (a: any) => {};
		uselessFn(modelId);
		return models?.list.find((x) => x.id.toString() == modelId);
	});
	let extensions = $derived(getSupportedFileExtensions(selectModelCap));

	const dropZone = createDropZone(() => container, {
		onDrop(newFiles: File[] | null) {
			if (newFiles == null) return;
			newFiles.forEach((f) => files.push(f));
		},
		onPaste(newFiles: File[] | null) {
			if (newFiles == null) return;
			newFiles.forEach((f) => files.push(f));
		}
	});

	afterNavigate((after) => {
		content = '';
		editable = true;
	});

	beforeNavigate((navigation) => {
		if (content.length == 0 && files.length == 0) return;

		const navigationUrl = navigation.to?.url.pathname || null;
		if (pendingNavigationUrl == navigationUrl) {
			showConfirm = false;
			pendingNavigationUrl = null;
		} else {
			navigation.cancel();
			showConfirm = true;
			pendingNavigationUrl = navigationUrl;
		}
	});

	function submit() {
		if (onsubmit && content.length > 0 && modelIdValid) {
			disabled = true;
			onsubmit();
		}
	}

	function handleContextMenu(event: MouseEvent) {
		event.preventDefault();
		if (content.trim().length === 0) {
			return;
		}
		showConvertToFileDialog = true;
	}

	function handleConvertToFile() {
		let fileName = convertFileName.trim();
		if (!fileName) {
			fileName = 'message';
		}

		if (!fileName.includes('.')) {
			fileName = fileName + '.md';
		}

		const blob = new Blob([content], { type: 'text/markdown' });
		const file = new File([blob], fileName, { type: 'text/markdown' });

		files.push(file);
		content = '';
		showConvertToFileDialog = false;
		convertFileName = '';
	}
</script>

<Confirm
	bind:open={showConfirm}
	onconfirm={() => {
		console.log(pendingNavigationUrl);
		goto(pendingNavigationUrl!);
	}}
/>

<div
	role="region"
	class="min-h-sm item relative mx-auto w-[90%] space-y-2 rounded-md border border-outline bg-chat-input-bg p-2 shadow-xl shadow-secondary md:w-[min(750px,75%)]"
	bind:this={container}
	oncontextmenu={handleContextMenu}
>
	{#if dropZone.isOver && editable}
		<div
			class="absolute top-0 left-0 flex h-full w-full items-center justify-center rounded-lg bg-primary text-2xl"
		>
			<FileUp />
			{$_('chat.upload_file')}
		</div>
	{/if}
	{#if files.length != 0}
		<div class="mb-2 max-h-[60vh] overflow-y-auto border-b border-outline pb-2">
			<FileGroup {files} {extensions} deletable />
		</div>
	{/if}
	<div class="flex flex-row items-center justify-between space-x-2 pr-2">
		<Textbox
			bind:editable
			placeholder={disabled ? $_('chat.stop_first') : $_('chat.question')}
			bind:value={content}
			onsubmit={submit}
			{disabled}
			minRow={large ? 2 : 1}
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
			<UploadBtn bind:files />
		</div>
		{#if content.length != 0}
			<MarkdownBtn bind:editable />
		{/if}
	</div>
</div>

<Modal bind:open={showConvertToFileDialog} title={$_('chat.convert_to_file.title')}>
	{#snippet children()}
		<div class="space-y-4">
			<p class="text-sm text-text/80">
				{$_('chat.convert_to_file.description')}
			</p>
			<div>
				<label for="filename" class="mb-2 block text-sm font-medium">
					{$_('chat.convert_to_file.filename_label')}
				</label>
				<input
					id="filename"
					type="text"
					bind:value={convertFileName}
					placeholder="message.md"
					class="w-full rounded-md border border-outline bg-chat-input-bg px-3 py-2 focus:ring-2 focus:ring-primary focus:outline-hidden"
				/>
				<p class="mt-1 text-xs text-text/60">
					{$_('chat.convert_to_file.file_hint')}
				</p>
			</div>
		</div>
	{/snippet}
	{#snippet footer()}
		<button
			onclick={() => {
				showConvertToFileDialog = false;
				convertFileName = '';
			}}
			class="rounded-md border border-outline bg-transparent px-4 py-2 transition-colors hover:bg-primary focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-hidden"
		>
			{$_('chat.convert_to_file.cancel')}
		</button>
		<button
			onclick={handleConvertToFile}
			class="rounded-md border border-outline bg-primary px-4 py-2 transition-colors hover:bg-primary/80 focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-hidden"
		>
			{$_('chat.convert_to_file.convert')}
		</button>
	{/snippet}
</Modal>
