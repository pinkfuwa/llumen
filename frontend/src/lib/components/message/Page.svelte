<script lang="ts">
	import type { PageEntry } from '$lib/api/state';
	import { MessagePaginateRespRole, type MessagePaginateRespList } from '$lib/api/types';
	import UserMessage from './User.svelte';
	import AssistantMessage from './Assistant.svelte';

	let div = $state<HTMLElement | null>(null);

	const { entry }: { entry: PageEntry<MessagePaginateRespList> } = $props();
	const data = entry.data;

	$effect(() => entry.target.set(div));
</script>

<div class="flex flex-col-reverse space-y-2" bind:this={div}>
	{#each $data as msg}
		{#if msg.role == MessagePaginateRespRole.User}
			<UserMessage content={msg.text} />
		{:else if msg.role == MessagePaginateRespRole.Assistant}
			<AssistantMessage content={msg.text} />
		{/if}
	{:else}
		<div class="h-1"></div>
	{/each}
</div>
