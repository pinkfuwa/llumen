<script lang="ts">
	import { CornerDownLeft, Book, BookA, ArrowUpDown } from 'lucide-svelte';
	import { render } from '$lib/markdown/markdown';

	let content = $state('');
	let enableMarkdown = $state(false);
</script>

<h1 class="mx-auto mb-6 text-4xl font-light">Ask anything</h1>
<div class="min-h-sm item mx-auto rounded-md border border-outline p-2 md:min-w-lg">
	<div class="mb-2 flex items-center justify-between border-b border-outline p-2 pb-4">
		{#if enableMarkdown}
			<div
				class="new-message markdown max-h-[60vh] max-w-[65vw] flex-grow space-y-2 overflow-scroll"
			>
				{#await render(content) then md}
					{@html md}
				{:catch someError}
					<div class="mb-4 flex items-center justify-center border-b border-outline p-6 text-lg">
						System error: {someError.message}.
					</div>
				{/await}
			</div>
		{:else}
			<div
				class="new-message editor max-h-[60vh] max-w-[65vw] flex-grow overflow-scroll"
				contenteditable="plaintext-only"
				placeholder="Type your question here..."
				bind:innerText={content}
			></div>
		{/if}

		<CornerDownLeft class="ml-2" />
	</div>
	<div class="flex flex-row justify-between">
		<div class="grow">
			<span class="inline-block items-center rounded-md bg-background px-3 py-1 hover:bg-hover"
				>Selected Model
				<ArrowUpDown class="ml-2 inline-block h-[20px] w-[20px]" />
			</span>
		</div>
		<div>
			{#if enableMarkdown}
				<BookA class="ml-2 inline-block" onclick={() => (enableMarkdown = false)} />
			{:else}
				<Book class="ml-2 inline-block" onclick={() => (enableMarkdown = true)} />
			{/if}
		</div>
	</div>
</div>

<style>
	.new-message:focus-visible {
		outline: none;
	}
	.editor::-webkit-scrollbar {
		display: none;
	}
	.markdown::-webkit-scrollbar {
		display: none;
	}
</style>
