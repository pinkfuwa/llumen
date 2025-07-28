<script lang="ts">
	let { editable = false, value = $bindable(''), placeholder = '' } = $props();

	import { renderMarkdown } from '$lib';

	let rows = () => Math.max(2, value.split('\n').length);
</script>

{#if editable}
	<textarea
		class="editor field-sizing-content max-h-[60vh] max-w-[65vw] flex-grow resize-none overflow-scroll"
		bind:value
		{placeholder}
		rows={rows()}
	></textarea>
{:else}
	<div
		class="new-message markdown max-h-[60vh] min-h-12 max-w-[65vw] flex-grow space-y-2 overflow-scroll wrap-break-word"
	>
		{#await renderMarkdown(value)}
			<div class="relative">
				<div class="mb-4 flex items-center justify-center p-6 text-lg">rendering markdown</div>
				<textarea class="h-full w-full resize-none opacity-0" disabled rows={rows()}></textarea>
			</div>
		{:then md}
			{@html md}
		{:catch someError}
			<div class="mb-4 flex items-center justify-center p-6 text-lg">
				System error: {someError.message}.
			</div>
		{/await}
	</div>
{/if}
