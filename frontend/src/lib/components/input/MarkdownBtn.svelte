<script lang="ts">
	import { Book, BookA } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import Tooltip from '$lib/ui/Tooltip.svelte';

	let { isEditing = $bindable(true), disabled = false } = $props();

	const markdownBtnStyle = 'hidden rounded-md p-1 hover:bg-interactive-hover md:block';
</script>

<Tooltip
	class={markdownBtnStyle}
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
