<script lang="ts">
	import Button from '$lib/ui/Button.svelte';
	import DangerButton from '$lib/ui/DangerButton.svelte';
	import Modal from '$lib/ui/Modal.svelte';
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
	import { AlertTriangle } from '@lucide/svelte';
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
	title={m['chat.unsupported_files.title'](lang)}
	onClose={uploadSupportedOnly}
>
	{#snippet children()}
		<div class="space-y-4">
			<p>
				{m['chat.unsupported_files.description'](lang)}
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
				{m['chat.unsupported_files.hint'](lang)}
			</p>
		</div>
	{/snippet}
	{#snippet footer()}
		<Button class="px-4 py-2" onclick={uploadSupportedOnly}>
			{m['chat.unsupported_files.upload_supported'](lang)}
		</Button>
		<DangerButton onclick={uploadAllFiles}>
			{m['chat.unsupported_files.upload_all'](lang)}
		</DangerButton>
	{/snippet}
</Modal>
