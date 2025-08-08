<script lang="ts">
	let { params } = $props();

	import { _ } from 'svelte-i18n';
	import MessageInput from '$lib/components/MessageInput.svelte';
	import MockMessage from '$lib/mockMessages.json';
	import UserMessage from '$lib/components/message/UserMessage.svelte';
	import AssistantMessage from '$lib/components/message/AssistantMessage.svelte';

	function scrollToBottom(node: HTMLElement) {
		node.scroll({ top: node.scrollHeight, behavior: 'smooth' });
	}
</script>

<div class="w-full grow space-y-2 pb-4">
	{#each MockMessage as msg, i}
		{#if i % 2 == 0}
			<UserMessage content={msg} />
		{:else}
			<AssistantMessage content={msg} />
		{/if}
	{/each}
	{#key params.id}
		<div use:scrollToBottom></div>
	{/key}
</div>
<div class="sticky bottom-2 mt-auto flex justify-center">
	<MessageInput above />
</div>
