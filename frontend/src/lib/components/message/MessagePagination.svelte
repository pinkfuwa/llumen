<script lang="ts">
	let { id, isStreaming = $bindable(false) }: { id: number; isStreaming?: boolean } = $props();

	import { handleServerSideMessage, useMessage } from '$lib/api/message';
	import Page from './Page.svelte';
	import type { TokensList } from 'marked';
	import MessageStream from './MessageStream.svelte';

	const { data } = useMessage(id);

	let tokensList = $state<Array<TokensList>>([]);

	handleServerSideMessage(id, {
		tick() {
			isStreaming = true;
		},
		reset() {
			tokensList = [];
			isStreaming = false;
		},
		append(tokens) {
			tokensList.push(tokens);
			isStreaming = true;
		},
		replace(tokens) {
			tokensList.pop();
			tokensList.push(tokens);
			isStreaming = true;
		}
	});
</script>

{#if isStreaming}
	<MessageStream list={tokensList} />
{/if}

{#each $data as page}
	{#key page.no}
		<Page entry={page} />
	{/key}
{/each}
