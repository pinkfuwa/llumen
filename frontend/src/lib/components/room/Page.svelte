<script lang="ts">
	import type { InfiniteQueryEntry } from '$lib/api/state';
	import { type ChatPaginateRespList } from '$lib/api/types';
	import ChatroomBtn from './ChatroomBtn.svelte';

	const {
		entry,
		currentRoom
	}: { entry: InfiniteQueryEntry<ChatPaginateRespList>; currentRoom: number | undefined } =
		$props();

	let li = $state<HTMLElement | null>(null);

	const data = entry.data;

	$effect(() => entry.target.set(li));
</script>

<li bind:this={li}>
	{#each $data as room}
		<ChatroomBtn name={room.title} id={room.id} selected={room.id == currentRoom} />
	{:else}
		<div class="h-1"></div>
	{/each}
</li>
