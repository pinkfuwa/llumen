<script lang="ts">
	import { Upload } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import TipButton from '$lib/ui/TipButton.svelte';

	let { files = $bindable([] as File[]) }: { files: File[] } = $props();

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

<TipButton
	class="aspect-square h-full shrink-0"
	onclick={openDialog}
	aria-label="upload file"
	text={$_('chat.file')}
>
	<Upload class="inline-block" />
</TipButton>

<input type="file" class="hidden" bind:this={inputElement} multiple={true} onchange={onChange} />
