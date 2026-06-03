<script lang="ts">
	import { MessageInput } from '$lib/components';
	import MessagePagination from '$lib/components/message/Pagination.svelte';
	import Hallucination from '$lib/components/common/Hallucination.svelte';
	import { _ } from 'svelte-i18n';
	import { messagesElement } from '$lib/api';
</script>

<Hallucination />

<div class="nobar anchor-none flex h-full flex-col overflow-y-auto" bind:this={messagesElement.val}>
	<div class="min-h-16 grow"></div>
	<MessagePagination />
	<div class="sticky bottom-1 z-10 mt-4 flex justify-center">
		<MessageInput />
	</div>
</div>
