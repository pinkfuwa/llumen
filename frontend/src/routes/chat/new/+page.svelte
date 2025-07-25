<script lang="ts">
	import {
		CornerDownLeft,
		Book,
		BookA,
		ArrowUpDown,
		Atom,
		SearchCode,
		ZapOff,
		Paperclip,
		X
	} from '@lucide/svelte';
	import { renderMarkdown } from '$lib';
	import { Tooltip } from '@svelte-plugins/tooltips';

	let content = $state('');
	let enableMarkdown = $state(false);

	// stage 0: normal
	// stage 1: search
	// stage 2: deep
	let stage = $state(0);
	let maxStage = $state(2);

	function nextStage() {
		stage = stage == maxStage ? 0 : stage + 1;
	}
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
	<div class="mb-2 flex items-center justify-between border-b border-outline p-2 pb-4">
		{#if enableMarkdown}
			<div
				class="new-message markdown max-h-[60vh] min-h-12 max-w-[65vw] flex-grow space-y-2 overflow-scroll"
			>
				{#await renderMarkdown(content)}
					<div class="mb-4 flex items-center justify-center p-6 text-lg">rendering markdown</div>
				{:then md}
					{@html md}
				{:catch someError}
					<div class="mb-4 flex items-center justify-center p-6 text-lg">
						System error: {someError.message}.
					</div>
				{/await}
			</div>
		{:else}
			<textarea
				class="editor field-sizing-content max-h-[60vh] max-w-[65vw] flex-grow resize-none overflow-scroll"
				placeholder="Type your question here..."
				bind:value={content}
			></textarea>
		{/if}

		<CornerDownLeft class="ml-2" />
	</div>
	<div class="flex flex-row items-center justify-between">
		<div class="flex grow items-center">
			<span class="inline-block items-center rounded-md bg-background px-3 py-1 hover:bg-hover"
				>Selected Model
				<ArrowUpDown class="ml-2 inline-block h-[20px] w-[20px]" />
			</span>
			<button onclick={nextStage} class="ml-1 rounded-md bg-background p-1 hover:bg-hover">
				{#if stage == 2}
					<Tooltip content="In depth research on complex topic">
						<Atom class="inline-block" />
					</Tooltip>
				{:else if stage == 1}
					<Tooltip content="Web enabled Chat">
						<SearchCode class="inline-block" />
					</Tooltip>
				{:else}
					<Tooltip content="Normal Chat">
						<ZapOff class="inline-block" />
					</Tooltip>
				{/if}
			</button>
		</div>
		<button
			class="ml-2 rounded-md p-1 hover:bg-hover"
			onclick={() => (enableMarkdown = !enableMarkdown)}
		>
			{#if enableMarkdown}
				<Tooltip content="Markdown rendering is enabled">
					<BookA />
				</Tooltip>
			{:else}
				<Tooltip content="Normal text editing">
					<Book />
				</Tooltip>
			{/if}
		</button>
	</div>
</div>

<style>
	.markdown::-webkit-scrollbar {
		display: none;
	}
</style>
