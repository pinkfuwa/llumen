<script lang="ts">
	import { _ } from 'svelte-i18n';
	import Page from './Page.svelte';
	import { useRoom } from '$lib/api/chatroom';
	import New from './New.svelte';

	let { addition = false, currentRoom = undefined as undefined | number } = $props();

	const { data } = useRoom();
</script>

<ul class="nobar max-h-[calc(100vh-185px)] space-y-1 overflow-y-auto text-sm">
	{#if addition}
		<li>
			<New />
		</li>
	{/if}
	{#each $data as page}
		{#key page.no}
			<Page entry={page} {currentRoom} />
		{/key}
	{/each}
</ul>
