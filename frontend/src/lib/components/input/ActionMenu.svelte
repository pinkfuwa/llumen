<script lang="ts">
	import { Plus, Upload, FileText, Mic } from '@lucide/svelte';
	import Dropdown from '$lib/ui/Dropdown.svelte';
	import Modal from '$lib/ui/Modal.svelte';
	import TipButton from '$lib/ui/TipButton.svelte';
	import { _ } from 'svelte-i18n';
	import { DropdownMenu } from 'bits-ui';
	import Button from '$lib/ui/Button.svelte';

	let {
		files = $bindable([]),
		content = $bindable(''),
		disabled = false,
		onFilesAdded
	}: {
		files: File[];
		content: string;
		disabled?: boolean;
		onFilesAdded?: (files: File[]) => void;
	} = $props();

	let dropdownOpen = $state(false);
	let showConvertToFileDialog = $state(false);
	let showRecordAudioDialog = $state(false);
	let convertFileName = $state('');
	let inputElement: HTMLInputElement | null = null;

	// Audio recording state
	let mediaRecorder: MediaRecorder | null = null;
	let audioChunks: Blob[] = [];
	let isRecording = $state(false);
	let recordingTime = $state(0);
	let recordingInterval: number | null = null;

	function openFileDialog() {
		dropdownOpen = false;
		inputElement?.click();
	}

	function onChange(event: Event) {
		const target = event.target as HTMLInputElement;
		if (target.files) {
			const newFiles = Array.from(target.files);
			if (onFilesAdded) {
				onFilesAdded(newFiles);
			} else {
				files = [...files, ...newFiles];
			}
		}
		if (inputElement) {
			inputElement.value = '';
		}
	}

	function openConvertDialog() {
		dropdownOpen = false;
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

	function openRecordDialog() {
		dropdownOpen = false;
		showRecordAudioDialog = true;
	}

	async function startRecording() {
		try {
			const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
			mediaRecorder = new MediaRecorder(stream);
			audioChunks = [];

			mediaRecorder.ondataavailable = (event) => {
				audioChunks.push(event.data);
			};

			mediaRecorder.onstop = () => {
				const audioBlob = new Blob(audioChunks, { type: 'audio/mp3' });
				const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
				const audioFile = new File([audioBlob], `recording-${timestamp}.mp3`, {
					type: 'audio/mp3'
				});
				files.push(audioFile);

				// Clean up
				stream.getTracks().forEach((track) => track.stop());
				if (recordingInterval) {
					clearInterval(recordingInterval);
					recordingInterval = null;
				}
			};

			mediaRecorder.start();
			isRecording = true;
			recordingTime = 0;

			recordingInterval = setInterval(() => {
				recordingTime += 1;
			}, 1000) as unknown as number;
		} catch (error) {
			console.error('Failed to start recording:', error);
		}
	}

	function stopRecording() {
		if (mediaRecorder && isRecording) {
			mediaRecorder.stop();
			isRecording = false;
			showRecordAudioDialog = false;
			recordingTime = 0;
		}
	}

	function cancelRecording() {
		if (mediaRecorder && isRecording) {
			mediaRecorder.stop();
			isRecording = false;
			audioChunks = [];
		}
		showRecordAudioDialog = false;
		recordingTime = 0;
		if (recordingInterval) {
			clearInterval(recordingInterval);
			recordingInterval = null;
		}
	}

	function formatTime(seconds: number): string {
		const mins = Math.floor(seconds / 60);
		const secs = seconds % 60;
		return `${mins}:${secs.toString().padStart(2, '0')}`;
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
			class="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm outline-hidden duration-150 select-none hover:bg-primary hover:text-text-hover"
			onSelect={openRecordDialog}
		>
			<Mic class="size-4" />
			<span>{$_('chat.record_audio')}</span>
		</DropdownMenu.Item>

		<DropdownMenu.Item
			class="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm outline-hidden duration-150 select-none hover:bg-primary hover:text-text-hover aria-disabled:text-text aria-disabled:opacity-50"
			onSelect={openConvertDialog}
			disabled={content.trim().length === 0}
			data-disabled={'true'}
		>
			<FileText class="size-4" />
			<span>{$_('chat.convert_to_file.action')}</span>
		</DropdownMenu.Item>
	{/snippet}
</Dropdown>

<input type="file" class="hidden" bind:this={inputElement} multiple={true} onchange={onChange} />

<Modal bind:open={showConvertToFileDialog} title={$_('chat.convert_to_file.title')}>
	{#snippet children()}
		<form
			class="space-y-4"
			onsubmit={(e) => {
				e.preventDefault();
				handleConvertToFile();
			}}
		>
			<p>
				{$_('chat.convert_to_file.description')}
			</p>
			<div>
				<label for="filename" class="mb-2 block">
					{$_('chat.convert_to_file.filename_label')}
				</label>
				<input
					id="filename"
					type="text"
					bind:value={convertFileName}
					placeholder="message.md"
					class="w-full rounded-md border border-outline bg-chat-input-bg px-3 py-2 focus:ring-2 focus:ring-primary focus:outline-hidden"
				/>
				<p class="mt-1 text-sm text-text/60">
					{$_('chat.convert_to_file.file_hint')}
				</p>
			</div>
		</form>
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

<Modal bind:open={showRecordAudioDialog} title={$_('chat.record_audio_dialog.title')}>
	{#snippet children()}
		<div class="space-y-4">
			<p>
				{$_('chat.record_audio_dialog.description')}
			</p>
			{#if isRecording}
				<div class="flex flex-col items-center gap-4 py-6">
					<div
						class="flex size-16 animate-pulse items-center justify-center rounded-full bg-red-500"
					>
						<Mic class="size-8 text-white" />
					</div>
					<div class="font-mono text-2xl font-semibold">
						{formatTime(recordingTime)}
					</div>
					<p class="text-sm text-text/60">
						{$_('chat.record_audio_dialog.recording')}
					</p>
				</div>
			{:else}
				<div class="flex flex-col items-center gap-4 py-6">
					<button
						onclick={startRecording}
						class="flex size-16 items-center justify-center rounded-full border-2 border-outline bg-primary transition-colors hover:bg-primary/80"
					>
						<Mic class="size-8" />
					</button>
					<p class="text-sm text-text/60">
						{$_('chat.record_audio_dialog.click_to_start')}
					</p>
				</div>
			{/if}
		</div>
	{/snippet}
	{#snippet footer()}
		<button
			onclick={cancelRecording}
			class="rounded-md border border-outline bg-transparent px-4 py-2 transition-colors hover:bg-primary focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-hidden"
		>
			{$_('chat.record_audio_dialog.cancel')}
		</button>
		{#if isRecording}
			<button
				onclick={stopRecording}
				class="rounded-md border border-outline bg-primary px-4 py-2 transition-colors hover:bg-primary/80 focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-hidden"
			>
				{$_('chat.record_audio_dialog.stop')}
			</button>
		{/if}
	{/snippet}
</Modal>
