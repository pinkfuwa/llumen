<script lang="ts">
	let { editable = false, value = $bindable(''), placeholder = '' } = $props();

	import { default as Markdown } from '../markdown/Root.svelte';

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
		<Markdown source={value} />
	</div>
{/if}
