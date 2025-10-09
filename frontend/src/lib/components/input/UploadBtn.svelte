<script lang="ts">
	let { files = $bindable([] as File[]), filetypes } = $props();
	import { Upload } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import Button from '$lib/ui/Button.svelte';
	import Tooltip from '../buttons/Tooltip.svelte';
	import { createFileDialog } from './fileDialog.svelte';
	import { onDestroy } from 'svelte';

	const dialog = $derived(
		createFileDialog({
			multiple: false,
			onChange(newfile) {
				files = [...files, newfile[0]];
			},
			accept: filetypes
		})
	);

	onDestroy(() => dialog.cleanup());
</script>

<Button class="aspect-square h-full shrink-0" onclick={dialog.open} aria-label="upload file">
	<Tooltip text={$_('chat.file')}>
		<Upload class="inline-block" />
	</Tooltip>
</Button>
