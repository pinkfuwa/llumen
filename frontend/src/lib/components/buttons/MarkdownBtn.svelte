<script lang="ts">
	import { Book, BookA } from '@lucide/svelte';
	import { Tooltip } from '@svelte-plugins/tooltips';

	let { editable = $bindable(true) } = $props();
</script>

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
