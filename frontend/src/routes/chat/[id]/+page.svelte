<script lang="ts">
	import { MessageInput, sidebarOpen } from '$lib/components';
	import MessagePagination from '$lib/components/message/Pagination.svelte';
	import Hallucination from '$lib/components/common/Hallucination.svelte';
	import { messagesElement } from '$lib/api';
</script>

<Hallucination />

<div
	class="nobar anchor-none flex h-full snap-y flex-col overflow-y-auto transition-all lg:data-widen:px-20"
	bind:this={messagesElement.val}
	data-widen={sidebarOpen.val ? undefined : ''}
>
	<div class="min-h-16 grow"></div>
	<MessagePagination />
	<div class="sticky bottom-1 z-10 mt-4 flex justify-center">
		<MessageInput />
	</div>
</div>
