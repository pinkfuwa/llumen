<script lang="ts">
	import type { InfiniteQueryEntry } from '$lib/api/state';
	import { MessagePaginateRespRole, type MessagePaginateRespList } from '$lib/api/types';
	import UserMessage from '$lib/components/message/UserMessage.svelte';
	import AssistantMessage from '$lib/components/message/AssistantMessage.svelte';

	let div = $state<HTMLElement | null>(null);

	const { entry }: { entry: InfiniteQueryEntry<MessagePaginateRespList> } = $props();
	const data = entry.data;

	$effect(() => entry.target.set(div));
</script>

<div class="space-y-2" bind:this={div}>
	{#each $data as msg}
		{#if msg.role == MessagePaginateRespRole.User}
			<UserMessage content={msg.text} />
		{:else if msg.role == MessagePaginateRespRole.Assistant}
			<AssistantMessage content={msg.text} />
		{/if}
	{/each}
</div>
