<script lang="ts">
	import Button from '$lib/ui/Button.svelte';
	import DangerButton from '$lib/ui/DangerButton.svelte';
	import Modal from '$lib/ui/Modal.svelte';
	import { _ } from 'svelte-i18n';
	import { AlertTriangle } from '@lucide/svelte';

	let {
		open = $bindable(false),
		unsupportedFiles = [],
		onUploadAll,
		onUploadSupported
	}: {
		open: boolean;
		unsupportedFiles: File[];
		onUploadAll: () => void;
		onUploadSupported: () => void;
	} = $props();
</script>

<Modal bind:open title={$_('chat.unsupported_files.title')}>
	{#snippet children()}
		<div class="space-y-4">
			<p>
				{$_('chat.unsupported_files.description')}
			</p>

			<div
				class="max-h-48 space-y-2 overflow-y-auto rounded-md border border-border bg-popover p-3"
			>
				{#each unsupportedFiles as file}
					<div class="flex items-center gap-2 rounded-md bg-primary/10 px-3 py-2 text-primary">
						<AlertTriangle class="size-6 shrink-0" />
						<span class="min-w-0 truncate">{file.name}</span>
					</div>
				{/each}
			</div>

			<p class="text-sm">
				{$_('chat.unsupported_files.hint')}
			</p>
		</div>
	{/snippet}
	{#snippet footer()}
		<Button class="px-4 py-2" onclick={onUploadSupported}>
			{$_('chat.unsupported_files.upload_supported')}
		</Button>
		<DangerButton onclick={onUploadAll}>
			{$_('chat.unsupported_files.upload_all')}
		</DangerButton>
	{/snippet}
</Modal>
