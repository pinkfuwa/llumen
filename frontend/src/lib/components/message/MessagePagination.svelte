<script lang="ts">
	let { id }: { id: number } = $props();

	import { _ } from 'svelte-i18n';
	import MockMessage from '$lib/mockMessages.json';
	import UserMessage from '$lib/components/message/UserMessage.svelte';
	import AssistantMessage from '$lib/components/message/AssistantMessage.svelte';

	function scrollToBottom(node: HTMLElement) {
		node.scroll({ top: node.scrollHeight, behavior: 'smooth' });
	}
</script>

<div class="space-y-2">
	{#each MockMessage as msg, i}
		{#if i % 2 == 0}
			<UserMessage
				content={msg}
				files={[
					{ name: 'manualA.pdf' },
					{ name: 'manualB.pdf' },
					{ name: 'manualC.pdf' },
					{ name: 'manualD.pdf' }
				]}
			/>
		{:else}
			<AssistantMessage content={msg} />
		{/if}
	{/each}
	{#key id}
		<div use:scrollToBottom></div>
	{/key}
</div>
