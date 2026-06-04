<script lang="ts">
	import { Upload } from '@lucide/svelte';
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
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
	text={m['chat.file'](lang)}
>
	<Upload class="inline-block" />
</TipButton>

<input type="file" class="hidden" bind:this={inputElement} multiple={true} onchange={onChange} />
