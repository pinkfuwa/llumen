<script lang="ts">
	import type { PageEntry } from '$lib/api/state';
	import { MessagePaginateRespRole as Role, type MessagePaginateRespList } from '$lib/api/types';
	import Root from '$lib/markdown/Root.svelte';
	import Reasoning from './buttons/Reasoning.svelte';
	import ResponseBox from './buttons/ResponseBox.svelte';
	import ResponseEdit from './buttons/ResponseEdit.svelte';
	import User from './buttons/User.svelte';

	let div = $state<HTMLElement | null>(null);

	const { entry }: { entry: PageEntry<MessagePaginateRespList> } = $props();
	const data = entry.data;

	const mergedData = $derived.by(() => {
		let result: Array<MessagePaginateRespList & { reasoning?: string }> = [];
		for (const entry of $data) {
			if (entry.role == Role.User) result.push(entry);
			else if (entry.role == Role.Assistant) result.push(entry);
			else if (entry.role == Role.Think) result.at(-1)!.reasoning = entry.text;
		}
		return result;
	});

	$effect(() => entry.target.set(div));
</script>

<div class="flex flex-col-reverse space-y-2" bind:this={div}>
	{#each mergedData as msg}
		{#if msg.role == Role.User}
			<User content={msg.text} />
		{:else if msg.role == Role.Assistant && msg.text.length != 0}
			<ResponseBox>
				{#if msg.reasoning != undefined}
					<Reasoning content={msg.reasoning} />
				{/if}
				<!--
				{#each msg.parts as part}
					{#if part.role == MessagePaginateRespPartRole.Response}
						<Root source={part.text} />
					{:else if part.role == MessagePaginateRespPartRole.Reasoning}
						<Reasoning content={part.text} />
					{:else if part.role == MessagePaginateRespPartRole.Reasoning}
						<Tool content={part.text} />
					{/if}
				{/each}
				<ResponseEdit />
				-->
				<Root source={msg.text} />
				<ResponseEdit />
			</ResponseBox>
		{/if}
	{:else}
		<div class="h-1"></div>
	{/each}
</div>
