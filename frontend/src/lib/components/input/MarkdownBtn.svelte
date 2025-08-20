<script lang="ts">
	import { Book, BookA } from '@lucide/svelte';
	import { Tooltip } from '@svelte-plugins/tooltips';
	import { _ } from 'svelte-i18n';

	let { editable = $bindable(true), disabled = false } = $props();
</script>

<button class="rounded-md p-1 hover:bg-hover" onclick={() => (editable = !editable)} {disabled}>
	{#if editable}
		<Tooltip content={$_('chat.markdown_mode.disable')}>
			<Book />
		</Tooltip>
	{:else}
		<Tooltip content={$_('chat.markdown_mode.enable')}>
			<BookA />
		</Tooltip>
	{/if}
</button>
