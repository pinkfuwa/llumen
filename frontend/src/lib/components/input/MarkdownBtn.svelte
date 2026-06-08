<script lang="ts">
	import { Book, BookA } from '@lucide/svelte';
	import Tooltip from '$lib/ui/Tooltip.svelte';
	import { t } from 'svelte-intl-precompile';
	import { isEditing } from './state.svelte';

	const markdownBtnStyle = 'hidden rounded-md p-1 hover:bg-interactive-hover md:block';
</script>

<Tooltip
	class={markdownBtnStyle}
	onclick={() => (isEditing.val = !isEditing.val)}
	text={isEditing.val ? $t('chat.markdown_mode.disable') : $t('chat.markdown_mode.enable')}
>
	{#if isEditing.val}
		<Book />
	{:else}
		<BookA />
	{/if}
</Tooltip>
