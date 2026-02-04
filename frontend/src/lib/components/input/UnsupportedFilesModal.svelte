<script lang="ts">
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
				class="bg-background max-h-48 space-y-2 overflow-y-auto rounded-md border border-outline p-3"
			>
				{#each unsupportedFiles as file}
					<div class="bg-hover flex items-center gap-2 rounded-md px-3 py-2">
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
		<button
			onclick={onUploadSupported}
			class="rounded-md border border-outline bg-transparent px-4 py-2 transition-colors hover:bg-primary focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-hidden"
		>
			{$_('chat.unsupported_files.upload_supported')}
		</button>
		<button
			onclick={onUploadAll}
			class="rounded-md border border-outline bg-primary px-4 py-2 transition-colors hover:bg-primary/80 focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-hidden"
		>
			{$_('chat.unsupported_files.upload_all')}
		</button>
	{/snippet}
</Modal>
