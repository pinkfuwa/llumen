<script lang="ts">
	let { addition = false, currentRoom = undefined as undefined | string } = $props();
	import ForwardPage from './Forward.svelte';
	import ChatroomBtn from './ChatroomBtn.svelte';
	import { useRecentRoom, useRoom } from '$lib/api/chatroom';
	import { Plus } from '@lucide/svelte';
	import { derived } from 'svelte/store';

	let div: HTMLElement | null = $state(null);
	const { data, nextParam } = useRoom(() => div);

	const page = 0;

	const recentData = useRecentRoom(() => $data?.[0].id);

	const firstPage = derived([data, recentData], ([$data, $recentData]) =>
		($data || []).filter((x) => !$recentData.map((y) => y.id).includes(x.id))
	);
</script>

<ul class="nobar max-h-[calc(100vh-185px)] overflow-y-auto text-sm">
	{#if addition}
		<li>
			<a
				href="/chat/new"
				class="mb-2 flex w-full items-center justify-center rounded-md border border-outline bg-light p-1.5 font-semibold hover:bg-hover"
			>
				<Plus class="mr-2 h-5 w-5" />
				New
			</a>
		</li>
	{/if}
	{#each $recentData.reverse() as room}
		<ChatroomBtn name={room.title} id={room.id} selected={room.id.toString() == currentRoom} />
	{/each}
	<div bind:this={div}>
		{#each $firstPage as room}
			<ChatroomBtn name={room.title} id={room.id} selected={room.id.toString() == currentRoom} />
		{/each}
	</div>
	{#if $nextParam}
		{#key page}
			<ForwardPage session={$nextParam} page={page + 1} {currentRoom} />
		{/key}
	{/if}
</ul>
