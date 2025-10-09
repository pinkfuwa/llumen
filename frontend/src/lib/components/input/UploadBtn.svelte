<script lang="ts">
	let { files = $bindable([] as File[]) } = $props();
	import { Upload } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import Button from '$lib/ui/Button.svelte';
	import Tooltip from '../buttons/Tooltip.svelte';
	import { getContext } from 'svelte';
	import type { Writable } from 'svelte/store';

	const filetypes = getContext<Writable<string>>('filetypes');

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
	accept={$filetypes}
	multiple={false}
	onchange={onChange}
/>
