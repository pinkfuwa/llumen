<script lang="ts">
	let { id }: { id: number } = $props();

	import { handleServerSideMessage, useMessage } from '$lib/api/message';
	import Page from './Page.svelte';
	import type { TokensList } from 'marked';
	import MessageStream from './MessageStream.svelte';

	const { data } = useMessage(id);

	let tokensList = $state<Array<TokensList & { monochrome?: boolean }>>([]);

	handleServerSideMessage(id, {
		reset() {
			tokensList = [];
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

{#if tokensList.length != 0}
	<MessageStream list={tokensList} />
{/if}

{#each $data as page}
	{#key page.no}
		<Page entry={page} />
	{/key}
{/each}
