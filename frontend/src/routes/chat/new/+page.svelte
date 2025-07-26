<script lang="ts">
	import MdTextbox from '$lib/components/MDTextbox.svelte';
	import SearchBtn from '$lib/components/buttons/SearchBtn.svelte';
	import UploadBtn from '$lib/components/buttons/UploadBtn.svelte';
	import SendBtn from '$lib/components/buttons/SendBtn.svelte';
	import FileGroup from '$lib/components/FileGroup.svelte';
	import ModelBtn from '$lib/components/buttons/ModelBtn.svelte';
	import MarkdownBtn from '$lib/components/buttons/MarkdownBtn.svelte';

	let stage = $state(0) as 0 | 1 | 2;
	let editable = $state(true);

	import { createFileDialog } from '@sv-use/core';

	let files = $state<File[]>([]);
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

<h1 class="mx-auto mb-8 text-4xl font-light lg:text-5xl">Ask anything</h1>
<div
	class="min-h-sm item mx-auto rounded-md border border-outline p-2 md:w-md lg:w-[calc(30vw+300px)] xl:w-[700px]"
>
	{#if files.length != 0}
		<div class="mb-2 overflow-scroll border-b border-outline pb-2">
			<FileGroup {files} />
		</div>
	{/if}
	<div class="mb-2 flex items-center justify-between space-x-2 border-b border-outline p-2 pb-4">
		<MdTextbox {editable} placeholder="Enter your question here" />
		<SendBtn />
	</div>
	<div class="flex flex-row items-center justify-between">
		<div class="flex grow items-center space-x-1">
			<ModelBtn />
			<SearchBtn bind:value={stage} />
			<UploadBtn onclick={() => dialog.open()} />
		</div>
		<MarkdownBtn bind:editable />
	</div>
</div>
