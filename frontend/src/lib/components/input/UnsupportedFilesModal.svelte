<script lang="ts">
	import Button from '$lib/ui/Button.svelte';
	import DangerButton from '$lib/ui/DangerButton.svelte';
	import Modal from '$lib/ui/Modal.svelte';
	import { AlertTriangle } from '@lucide/svelte';
	import { t } from 'svelte-intl-precompile';
	import {
		unsupportedFilesModalOpen,
		pendingUnsupportedFiles,
		allowedUnsupportedFiles,
		inputFiles
	} from './state.svelte';

	function uploadAllFiles() {
		for (const f of pendingUnsupportedFiles.val) inputFiles.val.push(f);
		allowedUnsupportedFiles.val = [...allowedUnsupportedFiles.val, ...pendingUnsupportedFiles.val];
		unsupportedFilesModalOpen.val = false;
		pendingUnsupportedFiles.val = [];
	}

	function uploadSupportedOnly() {
		unsupportedFilesModalOpen.val = false;
		pendingUnsupportedFiles.val = [];
	}
</script>

<Modal
	bind:open={unsupportedFilesModalOpen.val}
	title={$t('chat.unsupported_files.title')}
	onClose={uploadSupportedOnly}
>
	{#snippet children()}
		<div class="space-y-4">
			<p>
				{$t('chat.unsupported_files.description')}
			</p>

			<div
				class="max-h-48 space-y-2 overflow-y-auto rounded-md border border-border bg-popover p-3"
			>
				{#each pendingUnsupportedFiles.val as file}
					<div class="flex items-center gap-2 rounded-md bg-primary/10 px-3 py-2 text-primary">
						<AlertTriangle class="size-6 shrink-0" />
						<span class="min-w-0 truncate">{file.name}</span>
					</div>
				{/each}
			</div>

			<p class="text-sm">
				{$t('chat.unsupported_files.hint')}
			</p>
		</div>
	{/snippet}
	{#snippet footer()}
		<Button class="px-4 py-2" onclick={uploadSupportedOnly}>
			{$t('chat.unsupported_files.upload_supported')}
		</Button>
		<DangerButton onclick={uploadAllFiles}>
			{$t('chat.unsupported_files.upload_all')}
		</DangerButton>
	{/snippet}
</Modal>
