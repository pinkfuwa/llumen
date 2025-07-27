<script lang="ts">
	let { addition = false } = $props();

	import { observeIntersection } from '@sv-use/core';
	import { Plus, Ellipsis } from '@lucide/svelte';
	import ChatroomBtn from './buttons/ChatroomBtn.svelte';
	import { useRooms } from '$lib/api/chatroom';

	let roomlist = useRooms();

	let divNode = $state();

	observeIntersection(
		() => divNode! as HTMLElement | null | undefined,
		([entry]) => {
			if (entry?.isIntersecting && !$roomlist.isFetching) $roomlist.fetchNextPage();
		},
		{}
	);
</script>

<ul class="nobar max-h-[calc(100vh-185px)] overflow-y-scroll text-sm">
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
	{#each $roomlist.data as room}
		<a href="/chat/{encodeURIComponent(room.id)}">
			<ChatroomBtn name={room.title} />
		</a>
	{/each}

	{#if $roomlist.hasNextPage}
		<li bind:this={divNode}>
			<button
				onclick={() => $roomlist.fetchNextPage()}
				class="mb-2 flex w-full items-center justify-center rounded-md border border-outline bg-light p-1.5 font-semibold hover:bg-hover"
			>
				<Ellipsis class="h-5 w-5" />
			</button>
		</li>
	{/if}
</ul>
