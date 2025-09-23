<script lang="ts">
	import { goto } from '$app/navigation';
	import { deleteRoom, updateRoom } from '$lib/api/chatroom';
	import { addSSEHandler } from '$lib/api/message';
	import type { PageEntry } from '$lib/api/state';
	import { type ChatPaginateRespList } from '$lib/api/types';
	import { dispatchError } from '$lib/error';
	import { get } from 'svelte/store';
	import ChatroomBtn from './ChatroomBtn.svelte';

	const {
		entry,
		currentRoom
	}: { entry: PageEntry<ChatPaginateRespList>; currentRoom: number | undefined } = $props();

	let li = $state<HTMLElement | null>(null);

	const data = entry.data;

	const { mutate: update } = updateRoom();
	const { mutate: delete_ } = deleteRoom();

	$effect(() => entry.target.set(li));

	addSSEHandler('title', (resp) => {
		if (get(data).some((x) => x.id == currentRoom)) {
			data.update((list) => {
				const idx = list.findIndex((x) => x.id == currentRoom);
				if (idx !== -1) {
					list[idx].title = resp.title;
				}
				return list;
			});
		}
	});
</script>

<li bind:this={li} class="space-y-1">
	{#each $data as room, i}
		<ChatroomBtn
			name={room.title}
			id={room.id}
			selected={room.id == currentRoom}
			ondelete={() =>
				// TODO: delete is reserved keyword
				delete_({ id: room.id }, (resp) => {
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
			onupdate={(newTitle) =>
				update(
					{
						chat_id: room.id,
						title: newTitle
					},
					(res) => {
						if (res.wrote) {
							data.update((list) => {
								const idx = list.findIndex((x) => x.id == room.id);
								if (idx != -1) list[idx].title = newTitle;
								return list;
							});
						}
					}
				)}
		/>
	{:else}
		<div class="h-1"></div>
	{/each}
</li>
