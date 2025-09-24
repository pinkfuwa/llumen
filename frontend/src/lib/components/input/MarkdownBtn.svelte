<script lang="ts">
	import { Book, BookA } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import Tooltip from '../buttons/Tooltip.svelte';

	let { editable = $bindable(true), disabled = false } = $props();
</script>

<button class="hover:bg-hover rounded-md p-1" onclick={() => (editable = !editable)} {disabled}>
	{#if editable}
		<Tooltip text={$_('chat.markdown_mode.disable')}>
			<Book />
		</Tooltip>
	{:else}
		<Tooltip text={$_('chat.markdown_mode.enable')}>
			<BookA />
		</Tooltip>
	{/if}
</button>
