<script lang="ts">
	import { Plus, Upload, FileText, Mic } from '@lucide/svelte';
	import Dropdown from '$lib/ui/Dropdown.svelte';
	import { _ } from 'svelte-i18n';
	import { DropdownMenu } from 'bits-ui';
	import { ChatMode as Mode } from '$lib/api/types';
	import type { ModelList } from '$lib/api/types';
	import ConvertToFileModal from './ConvertToFileModal.svelte';
	import RecordAudioModal from './RecordAudioModal.svelte';
	import ModeSelector from './ModeSelector.svelte';

	let {
		files = $bindable([]),
		content = $bindable(''),
		mode = $bindable(Mode.Normal),
		modelCap = undefined,
		onFilesAdded
	}: {
		files: File[];
		content: string;
		mode: Mode;
		modelCap?: ModelList;
		onFilesAdded?: (files: File[]) => void;
	} = $props();

	$inspect('content', content);

	let dropdownOpen = $state(false);
	let showConvertToFileDialog = $state(false);
	let showRecordAudioDialog = $state(false);
	let inputElement: HTMLInputElement | null = null;

	function openFileDialog() {
		dropdownOpen = false;
		inputElement?.click();
	}

	function onChange(event: Event) {
		const target = event.target as HTMLInputElement;
		if (target.files) {
			addFiles(Array.from(target.files));
		}
		if (inputElement) {
			inputElement.value = '';
		}
	}

	function addFiles(newFiles: File[]) {
		if (onFilesAdded) {
			onFilesAdded(newFiles);
		} else {
			files = [...files, ...newFiles];
		}
	}

	function openConvertDialog() {
		dropdownOpen = false;
		if (content.trim().length === 0) {
			return;
		}
		showConvertToFileDialog = true;
	}

	function openRecordDialog() {
		dropdownOpen = false;
		showRecordAudioDialog = true;
	}
</script>

<Dropdown bind:open={dropdownOpen}>
	{#snippet trigger()}
		<Plus class="inline-block" />
	{/snippet}
	{#snippet children()}
		<DropdownMenu.Item
			class="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm outline-hidden duration-150 select-none hover:bg-primary hover:text-text-hover"
			onSelect={openFileDialog}
		>
			<Upload class="size-4" />
			<span>{$_('chat.upload_file')}</span>
		</DropdownMenu.Item>

		<DropdownMenu.Item
			class="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm outline-hidden duration-150 select-none hover:bg-primary hover:text-text-hover aria-disabled:text-text aria-disabled:opacity-50"
			onSelect={openRecordDialog}
			disabled={!modelCap?.audio_input}
			data-disabled={!modelCap?.audio_input ? 'true' : 'false'}
		>
			<Mic class="size-4" />
			<span>{$_('chat.record_audio')}</span>
		</DropdownMenu.Item>

		<DropdownMenu.Item
			class="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm outline-hidden duration-150 select-none hover:bg-primary hover:text-text-hover aria-disabled:text-text aria-disabled:opacity-50"
			onSelect={openConvertDialog}
			disabled={content.trim().length === 0}
			data-disabled={content.trim().length === 0 ? 'true' : 'false'}
		>
			<FileText class="size-4" />
			<span>{$_('chat.convert_to_file.action')}</span>
		</DropdownMenu.Item>

		<ModeSelector bind:value={mode} {modelCap} />
	{/snippet}
</Dropdown>

<input type="file" class="hidden" bind:this={inputElement} multiple={true} onchange={onChange} />

<ConvertToFileModal bind:open={showConvertToFileDialog} bind:content onAddFiles={addFiles} />

<RecordAudioModal bind:open={showRecordAudioDialog} onAddFiles={addFiles} />
