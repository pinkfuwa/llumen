<script lang="ts">
	import { Book, BookA } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import Tooltip from '$lib/ui/Tooltip.svelte';

	let { isEditing = $bindable(true), disabled = false } = $props();
</script>

<Tooltip
	class="hover:bg-hover hidden rounded-md p-1 md:block"
	onclick={() => (isEditing = !isEditing)}
	text={isEditing ? $_('chat.markdown_mode.disable') : $_('chat.markdown_mode.enable')}
	{disabled}
>
	{#if isEditing}
		<Book />
	{:else}
		<BookA />
	{/if}
</Tooltip>
