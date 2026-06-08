<script lang="ts">
	import { Plus, Upload, FileText, Mic } from '@lucide/svelte';
	import Dropdown from '$lib/ui/Dropdown.svelte';
	import { DropdownMenu } from 'bits-ui';
	import ConvertToFileModal from './ConvertToFileModal.svelte';
	import RecordAudioModal from './RecordAudioModal.svelte';
	import ModeSelector from './ModeSelector.svelte';
	import { inputContent, effective, addFiles } from './state.svelte';
	import { t } from 'svelte-intl-precompile';

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

	function handleAddFiles(newFiles: File[]) {
		addFiles(newFiles);
	}

	function openConvertDialog() {
		dropdownOpen = false;
		if (inputContent.val.trim().length === 0) return;
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
			class="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm outline-hidden duration-150 select-none hover:bg-interactive-hover"
			onSelect={openFileDialog}
		>
			<Upload class="size-4" />
			<span>{$t('chat.upload_file')}</span>
		</DropdownMenu.Item>

		<DropdownMenu.Item
			class="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm outline-hidden duration-150 select-none hover:bg-interactive-hover aria-disabled:text-foreground aria-disabled:opacity-50"
			onSelect={openRecordDialog}
			disabled={!effective.currentModel?.audio_input}
			data-disabled={!effective.currentModel?.audio_input ? 'true' : 'false'}
		>
			<Mic class="size-4" />
			<span>{$t('chat.record_audio')}</span>
		</DropdownMenu.Item>

		<DropdownMenu.Item
			class="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm outline-hidden duration-150 select-none hover:bg-interactive-hover aria-disabled:text-foreground aria-disabled:opacity-50"
			onSelect={openConvertDialog}
			disabled={inputContent.val.trim().length === 0}
			data-disabled={inputContent.val.trim().length === 0 ? 'true' : 'false'}
		>
			<FileText class="size-4" />
			<span>{$t('chat.convert_to_file.action')}</span>
		</DropdownMenu.Item>

		<ModeSelector />
	{/snippet}
</Dropdown>

<input type="file" class="hidden" bind:this={inputElement} multiple={true} onchange={onChange} />

<ConvertToFileModal
	bind:open={showConvertToFileDialog}
	bind:content={inputContent.val}
	onAddFiles={handleAddFiles}
/>

<RecordAudioModal bind:open={showRecordAudioDialog} onAddFiles={handleAddFiles} />
