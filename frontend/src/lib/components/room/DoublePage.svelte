<script lang="ts">
	import Self from './DoublePage.svelte';
	let { session = new RoomSession(), depth = 0 } = $props();
	import ChatroomBtn from './ChatroomBtn.svelte';
	import { usePagedRoom, RoomSession } from '$lib/api/chatroom';
	import { onMount } from 'svelte';

	let { data } = usePagedRoom(session);

	let nextSession = $derived($data == undefined ? undefined : $data!.nextSession);

	onMount(() => {
		console.log(`Mounting component at depth: ${depth}`);
		console.log(`usePagedRoom called for session: ${JSON.stringify(session)} at depth: ${depth}`);
		return () => {
			console.log(`Unmounting component at depth: ${depth}`);
		};
	});
</script>

<div>
	{#if $data && Array.isArray($data.list)}
		{#each $data.list as room}
			<a href="/chat/{encodeURIComponent(room.id)}">
				<ChatroomBtn name={room.title} />
			</a>
		{/each}
	{/if}
</div>
{#if depth < 4 && nextSession}
	<Self session={nextSession} depth={depth + 1} />
{/if}
