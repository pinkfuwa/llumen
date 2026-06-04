<script lang="ts">
	import { Book, BookA } from '@lucide/svelte';
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
	import Tooltip from '$lib/ui/Tooltip.svelte';

	let { isEditing = $bindable(true), disabled = false } = $props();

	const markdownBtnStyle = 'hidden rounded-md p-1 hover:bg-interactive-hover md:block';
</script>

<Tooltip
	class={markdownBtnStyle}
	onclick={() => (isEditing = !isEditing)}
	text={isEditing ? m['chat.markdown_mode.disable'](lang) : m['chat.markdown_mode.enable'](lang)}
	{disabled}
>
	{#if isEditing}
		<Book />
	{:else}
		<BookA />
	{/if}
</Tooltip>
