<script lang="ts">
	import Button from '$lib/ui/Button.svelte';
	import DangerButton from '$lib/ui/DangerButton.svelte';
	import Modal from '$lib/ui/Modal.svelte';
	import { t } from 'svelte-intl-precompile';

	interface Props {
		open?: boolean;
		content: string;
		onAddFiles: (files: File[]) => void;
	}

	let { open = $bindable(false), content = $bindable(''), onAddFiles }: Props = $props();
	let convertFileName = $state('');

	const inputStyle = '';

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

<Modal bind:open title={$t('chat.convert_to_file.title')} onClose={close}>
	{#snippet children()}
		<form
			class="space-y-4"
			onsubmit={(event) => {
				event.preventDefault();
				handleConvertToFile();
			}}
		>
			<p>
				{$t('chat.convert_to_file.description')}
			</p>
			<div>
				<label for="filename" class="mb-2 block">
					{$t('chat.convert_to_file.filename_label')}
				</label>
				<input
					id="filename"
					type="text"
					bind:value={convertFileName}
					placeholder="message.md"
					class="w-full rounded-md border border-border bg-card px-3 py-2 focus:ring-2 focus:ring-accent focus:outline-hidden"
				/>
				<p class="mt-1 text-sm text-muted-foreground">
					{$t('chat.convert_to_file.file_hint')}
				</p>
			</div>
		</form>
	{/snippet}
	{#snippet footer()}
		<Button class="px-4 py-2" onclick={close}>
			{$t('chat.convert_to_file.cancel')}
		</Button>
		<DangerButton onclick={handleConvertToFile}>
			{$t('chat.convert_to_file.convert')}
		</DangerButton>
	{/snippet}
</Modal>
