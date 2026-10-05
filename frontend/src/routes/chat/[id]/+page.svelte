<script lang="ts">
	import { chatrooms } from '$lib/api';
	import { MessageInput, sidebarOpen, Minimap } from '$lib/components';
	import MessagePagination from '$lib/components/message/Pagination.svelte';
	import Hallucination from '$lib/components/common/Hallucination.svelte';
	import { messagesElement } from '$lib/api';
	import { page } from '$app/state';
	import { t } from 'svelte-intl-precompile';
	import ChatViewport from '$lib/ui/ChatViewport.svelte';

	let title = $derived.by(() => {
		const id = page.params.id;
		if (!id) return $t('chat.title');
		return chatrooms.val.find((e) => e.id === Number(id))?.name ?? $t('chat.title');
	});
</script>

<svelte:head>
	<title>
		{title}
	</title>
</svelte:head>

<Hallucination />
<Minimap />

<ChatViewport bind:element={messagesElement.val} widen={!sidebarOpen.val}>
	<MessagePagination />
	{#snippet composer()}
		<MessageInput />
	{/snippet}
</ChatViewport>
