<script lang="ts">
	import { createDropZone } from './dropzone.svelte';
	import Textbox from './Textbox.svelte';
	import ModeSelector from './ModeSelector.svelte';
	import ActionMenu from './ActionMenu.svelte';
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
		if (onsubmit && (content.length > 0 || files.length > 0) && modelIdValid) {
			disabled = true;
			onsubmit();
		}
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
			<Send
				onclick={submit}
				disabled={(content.length == 0 && files.length == 0) || !modelIdValid}
			/>
		{/if}
	</div>
	<div class="flex flex-row items-center justify-between">
		<div class="flex h-11 w-full grow items-center justify-start space-x-2">
			<ModelSelector bind:value={modelId} />
			<ModeSelector bind:value={mode} limited={!selectModelCap?.tool} />
			<ActionMenu bind:files bind:content {disabled} />
		</div>
		{#if content.length != 0}
			<MarkdownBtn bind:editable />
		{/if}
	</div>
</div>
