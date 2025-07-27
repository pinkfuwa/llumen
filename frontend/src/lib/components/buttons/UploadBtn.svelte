<script lang="ts">
	import { Tooltip } from '@svelte-plugins/tooltips';
	import { Upload } from '@lucide/svelte';
	import { createFileDialog } from '@sv-use/core';
	let { files = $bindable([] as File[]) } = $props();

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

<button class="rounded-md bg-background p-1 hover:bg-hover" onclick={dialog.open}>
	<Tooltip content="Upload file">
		<Upload class="inline-block" />
	</Tooltip>
</button>
