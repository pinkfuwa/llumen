<script lang="ts">
	let { id }: { id: number } = $props();

	import { handleServerSideMessage, useMessage } from '$lib/api/message';
	import Page from './Page.svelte';
	import type { TokensList } from 'marked';
	import MessageStream from './MessageStream.svelte';
	import { useRoomStreamingState } from '$lib/api/chatroom';

	const { data } = useMessage(id);

	let tokensList = $state<Array<TokensList>>([]);

	let isStreaming = $derived(useRoomStreamingState(id));

	handleServerSideMessage(id, {
		tick() {
			isStreaming.set(true);
		},
		reset() {
			tokensList = [];
			isStreaming.set(false);
		},
		append(tokens) {
			tokensList.push(tokens);
		},
		replace(tokens) {
			tokensList.pop();
			tokensList.push(tokens);
		}
	});
</script>

{#if $isStreaming}
	<MessageStream list={tokensList} />
{/if}

{#each $data as page}
	{#key page.no}
		<Page entry={page} />
	{/key}
{/each}
