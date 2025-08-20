<script lang="ts">
	let { files = $bindable([] as File[]) } = $props();
	import { Tooltip } from '@svelte-plugins/tooltips';
	import { Upload } from '@lucide/svelte';
	import { createFileDialog } from '@sv-use/core';
	import { _ } from 'svelte-i18n';

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

<button class="rounded-md bg-primary p-1 hover:bg-hover" onclick={dialog.open}>
	<Tooltip content={$_('chat.file')}>
		<Upload class="inline-block" />
	</Tooltip>
</button>
