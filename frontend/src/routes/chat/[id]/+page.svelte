<script lang="ts">
	let { params } = $props();
	import { _ } from 'svelte-i18n';
	import MessageInput from '$lib/components/MessageInput.svelte';
	import MessagePagination from '$lib/components/message/MessagePagination.svelte';
</script>

<div class="w-full grow pb-4">
	<MessagePagination id={Number(params.id)} />
</div>
<div class="sticky bottom-2 mt-auto flex justify-center">
	<MessageInput above />
</div>
