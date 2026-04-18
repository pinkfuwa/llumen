<script lang="ts">
	import Modal from '$lib/ui/Modal.svelte';
	import { _ } from 'svelte-i18n';

	interface Props {
		open?: boolean;
		content: string;
		onAddFiles: (files: File[]) => void;
	}

	let { open = $bindable(false), content = $bindable(''), onAddFiles }: Props = $props();
	let convertFileName = $state('');

	function close() {
		open = false;
		convertFileName = '';
	}

	function handleConvertToFile() {
		let fileName = convertFileName.trim();
		if (!fileName) {
			fileName = 'message';
		}

		if (!fileName.includes('.')) {
			fileName = `${fileName}.md`;
		}

		const blob = new Blob([content], { type: 'text/markdown' });
		const file = new File([blob], fileName, { type: 'text/markdown' });

		onAddFiles([file]);
		content = '';
		close();
	}
</script>

<Modal bind:open title={$_('chat.convert_to_file.title')} onClose={close}>
	{#snippet children()}
		<form
			class="space-y-4"
			onsubmit={(event) => {
				event.preventDefault();
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
			onclick={close}
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
