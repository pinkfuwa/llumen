<script lang="ts">
	import Self from './ForwardPage.svelte';
	let { session = undefined, page = 0 } = $props();
	import ChatroomBtn from './ChatroomBtn.svelte';
	import { useForwardRoom } from '$lib/api/chatroom';

	let div: HTMLElement | null = $state(null);
	let { data, nextParam } = useForwardRoom(() => div, session);
</script>

<div bind:this={div}>
	{#each $data || [] as room}
		<a href="/chat/{encodeURIComponent(room.id)}">
			<ChatroomBtn name={room.title} />
		</a>
	{/each}
</div>
{#if $nextParam}
	{#key page}
		<Self session={$nextParam} page={page + 1} />
	{/key}
{/if}
