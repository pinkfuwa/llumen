<script lang="ts">
	import { goto } from '$app/navigation';
	import { deleteRoom } from '$lib/api/chatroom';
	import type { PageEntry } from '$lib/api/state';
	import { type ChatPaginateRespList } from '$lib/api/types';
	import { dispatchError } from '$lib/error';
	import ChatroomBtn from './ChatroomBtn.svelte';

	const {
		entry,
		currentRoom
	}: { entry: PageEntry<ChatPaginateRespList>; currentRoom: number | undefined } = $props();

	let li = $state<HTMLElement | null>(null);

	const data = entry.data;

	$effect(() => entry.target.set(li));
</script>

<li bind:this={li} class="space-y-1">
	{#each $data as room, i}
		<ChatroomBtn
			name={room.title}
			id={room.id}
			selected={room.id == currentRoom}
			ondelete={() =>
				deleteRoom().mutate({ id: room.id }, (resp) => {
					if (!resp.deleted) {
						dispatchError('network', 'Fail to delete');
						return;
					}
					data.update((x) => {
						x.splice(i, 1);
						return x;
					});
					if (room.id == currentRoom) goto('/chat/new');
				})}
		/>
	{:else}
		<div class="h-1"></div>
	{/each}
</li>
