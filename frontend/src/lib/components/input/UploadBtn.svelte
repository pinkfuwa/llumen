<script lang="ts">
	let { files = $bindable([] as File[]), filetypes } = $props();
	import { Tooltip } from '@svelte-plugins/tooltips';
	import { Upload } from '@lucide/svelte';
	import { createFileDialog } from '@sv-use/core';
	import { _ } from 'svelte-i18n';
	import Button from '$lib/ui/Button.svelte';

	const dialog = $derived(
		createFileDialog({
			multiple: false,
			onChange(newfile) {
				files = [...files, newfile[0]];
			},
			accept: filetypes
		})
	);
</script>

<Button class="aspect-square h-full" onclick={dialog.open} aria-label="upload file">
	<Tooltip content={$_('chat.file')}>
		<Upload class="inline-block" />
	</Tooltip>
</Button>
