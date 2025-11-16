<script lang="ts">
	import { Upload } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import Button from '$lib/ui/Button.svelte';
	import Tooltip from '../buttons/Tooltip.svelte';

	let { files = $bindable([] as File[]), filetypes }: { files: File[]; filetypes: string } =
		$props();

	let inputElement: HTMLInputElement | null = null;

	function openDialog() {
		inputElement?.click();
	}

	function onChange(event: Event) {
		const target = event.target as HTMLInputElement;
		if (target.files) {
			files = [...files, ...Array.from(target.files)];
		}
		if (inputElement) {
			inputElement.value = '';
		}
	}
</script>

<Button class="aspect-square h-full shrink-0" onclick={openDialog} aria-label="upload file">
	<Tooltip text={$_('chat.file')}>
		<Upload class="inline-block" />
	</Tooltip>
</Button>

<input
	type="file"
	class="hidden"
	bind:this={inputElement}
	accept={filetypes}
	multiple={false}
	onchange={onChange}
/>
