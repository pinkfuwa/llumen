<script lang="ts">
	let { files = $bindable([] as File[]) } = $props();
	import { Tooltip } from '@svelte-plugins/tooltips';
	import { Upload } from '@lucide/svelte';
	import { createFileDialog } from '@sv-use/core';
	import { _ } from 'svelte-i18n';
	import Button from '$lib/ui/Button.svelte';

	const dialog = createFileDialog({
		multiple: false,
		onChange(newfile) {
			files = [...files, newfile[0]];
		},
		onCancel() {
			console.log('cancelled');
		}
	});
</script>

<Button class="aspect-square h-full" onclick={dialog.open} aria-label="upload file">
	<Tooltip content={$_('chat.file')}>
		<Upload class="inline-block" />
	</Tooltip>
</Button>
