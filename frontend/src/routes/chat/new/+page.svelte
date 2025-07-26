<script lang="ts">
	import { Book, BookA, ArrowUpDown, Paperclip, X } from '@lucide/svelte';
	import { Tooltip } from '@svelte-plugins/tooltips';
	import MdTextbox from '$lib/components/MDTextbox.svelte';
	import SearchBtn from '$lib/components/buttons/SearchBtn.svelte';
	import UploadBtn from '$lib/components/buttons/UploadBtn.svelte';
	import SendBtn from '$lib/components/buttons/SendBtn.svelte';

	// stage 0: normal
	// stage 1: search
	// stage 2: deep
	let stage = $state(0);
	let editable = $state(true);
</script>

<h1 class="mx-auto mb-8 text-4xl font-light lg:text-5xl">Ask anything</h1>
<div
	class="min-h-sm item mx-auto rounded-md border border-outline p-2 md:w-md lg:w-[calc(30vw+300px)] xl:w-[700px]"
>
	<div class="mb-2 flex items-center justify-between border-b border-outline pb-2">
		<div class="group relative rounded-md border border-outline bg-background p-2 hover:bg-hover">
			<Paperclip class="absolute top-2 left-11 h-10 w-10 opacity-20" />
			<div class="flex h-10 w-28 items-center justify-center">File 1</div>
			<X
				class="absolute top-0 right-0 hidden h-5 w-5 rounded-sm border border-outline bg-background p-[2px] group-hover:block"
			/>
		</div>
	</div>
	<div class="mb-2 flex items-center justify-between space-x-2 border-b border-outline p-2 pb-4">
		<MdTextbox {editable} placeholder="Enter your question here" />
		<SendBtn />
	</div>
	<div class="flex flex-row items-center justify-between">
		<div class="flex grow items-center space-x-1">
			<span class="inline-block items-center rounded-md bg-background px-3 py-1 hover:bg-hover"
				>Selected Model
				<ArrowUpDown class="ml-2 inline-block h-[20px] w-[20px]" />
			</span>
			<SearchBtn value={stage} />
			<UploadBtn />
		</div>
		<button class="rounded-md p-1 hover:bg-hover" onclick={() => (editable = !editable)}>
			{#if editable}
				<Tooltip content="Normal text editing">
					<Book />
				</Tooltip>
			{:else}
				<Tooltip content="Markdown rendering is enabled">
					<BookA />
				</Tooltip>
			{/if}
		</button>
	</div>
</div>
