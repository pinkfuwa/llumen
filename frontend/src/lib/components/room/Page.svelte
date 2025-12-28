<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { deleteRoom, updateRoom, getRoomPages, setRoomPages } from '$lib/api/chatroom.svelte';
	import type { PageState } from '$lib/api/state';
	import { type ChatPaginateRespList } from '$lib/api/types';
	import { dispatchError } from '$lib/error';
	import ChatroomEntry from './ChatroomEntry.svelte';
	import { _ } from 'svelte-i18n';

	const { entry }: { entry: PageState<ChatPaginateRespList> } = $props();

	let li = $state<HTMLElement | null>(null);

	const { mutate: update } = updateRoom();
	const { mutate: delete_ } = deleteRoom();

	const currentRoom = $derived.by(() => {
		if (page.route.id != '/chat/[id]') return;
		return parseInt(page.params.id!);
	});
	console.log();

	$effect(() => {
		if (li !== entry.target) {
			const pages = getRoomPages();
			const pageIndex = pages.findIndex((p) => p.no === entry.no);
			if (pageIndex !== -1) {
				const updatedPages = [...pages];
				updatedPages[pageIndex] = { ...updatedPages[pageIndex], target: li };
				setRoomPages(updatedPages);
			}
		}
	});
</script>

<li bind:this={li} class="space-y-1">
	{#each entry.data as room, i}
		<ChatroomEntry
			name={room.title}
			id={room.id}
			selected={room.id == currentRoom}
			ondelete={() =>
				// TODO: delete is reserved keyword
				delete_({ id: room.id }, (resp) => {
					if (!resp.deleted) {
						dispatchError('network', $_('error.fail_to_delete'));
						return;
					}
					const pages = getRoomPages();
					const pageIndex = pages.findIndex((p) => p.no === entry.no);
					if (pageIndex !== -1) {
						const updatedPages = [...pages];
						const updatedData = [...updatedPages[pageIndex].data];
						updatedData.splice(i, 1);
						updatedPages[pageIndex] = { ...updatedPages[pageIndex], data: updatedData };
						setRoomPages(updatedPages);
					}
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
							const pages = getRoomPages();
							const pageIndex = pages.findIndex((p) => p.no === entry.no);
							if (pageIndex !== -1) {
								const updatedPages = [...pages];
								const updatedData = updatedPages[pageIndex].data.map((r) =>
									r.id === room.id ? { ...r, title: newTitle } : r
								);
								updatedPages[pageIndex] = { ...updatedPages[pageIndex], data: updatedData };
								setRoomPages(updatedPages);
							}
						}
					}
				)}
		/>
	{:else}
		<div class="h-1"></div>
	{/each}
</li>
