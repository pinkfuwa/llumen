<script lang="ts">
	import type { PageEntry } from '$lib/api/state';
	import { MessagePaginateRespRole, type MessagePaginateRespList } from '$lib/api/types';
	import Root from '$lib/markdown/Root.svelte';
	import ResponseBox from './buttons/ResponseBox.svelte';
	import ResponseEdit from './buttons/ResponseEdit.svelte';
	import ResponseError from './buttons/ResponseError.svelte';
	import User from './buttons/User.svelte';

	let div = $state<HTMLElement | null>(null);

	const { entry }: { entry: PageEntry<MessagePaginateRespList> } = $props();
	const data = entry.data;

	$effect(() => entry.target.set(div));
</script>

<div class="flex flex-col-reverse space-y-2" bind:this={div}>
	{#each $data as msg}
		{#if msg.role == MessagePaginateRespRole.User}
			<User content={msg.text} />
		{:else if msg.role == MessagePaginateRespRole.Assistant}
			<ResponseBox>
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
