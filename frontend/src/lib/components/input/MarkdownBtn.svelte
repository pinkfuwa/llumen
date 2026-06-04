<script lang="ts">
	import { Book, BookA } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Tooltip from '$lib/ui/Tooltip.svelte';

	let { isEditing = $bindable(true), disabled = false } = $props();

	const markdownBtnStyle = 'hidden rounded-md p-1 hover:bg-interactive-hover md:block';
</script>

<Tooltip
	class={markdownBtnStyle}
	onclick={() => (isEditing = !isEditing)}
	text={isEditing ? m['chat.markdown_mode.disable']() : m['chat.markdown_mode.enable']()}
	{disabled}
>
	{#if isEditing}
		<Book />
	{:else}
		<BookA />
	{/if}
</Tooltip>
