<script lang="ts">
	import { chatrooms } from '$lib/api';
	import { MessageInput, sidebarOpen, Minimap } from '$lib/components';
	import MessagePagination from '$lib/components/message/Pagination.svelte';
	import Hallucination from '$lib/components/common/Hallucination.svelte';
	import { messagesElement } from '$lib/api';
	import { page } from '$app/state';
	import { t } from 'svelte-intl-precompile';

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

<div
	class="nobar anchor-none relative flex h-full flex-col overflow-y-auto transition-all lg:data-widen:px-24"
	bind:this={messagesElement.val}
	data-widen={sidebarOpen.val ? undefined : ''}
>
	<div class="min-h-16 grow snap-start"></div>
	<MessagePagination />

	<div class="sticky bottom-1 z-10 mt-4 flex snap-end justify-center">
		<MessageInput />
	</div>
</div>
