<script lang="ts">
	import Self from './Forward.svelte';
	let { session = undefined, page = 0 } = $props();
	import ChatroomBtn from './ChatroomBtn.svelte';
	import { useRoom } from '$lib/api/chatroom';
	import { LoaderCircle } from '@lucide/svelte';

	let div: HTMLElement | null = $state(null);
	let { data, nextParam } = useRoom(() => div, session);
</script>

<div bind:this={div}>
	{#if $data != undefined}
		{#each $data as room}
			<a href="/chat/{encodeURIComponent(room.id)}">
				<ChatroomBtn name={room.title} />
			</a>
		{/each}
	{:else}
		<div class="mt-1 flex justify-center">
			<LoaderCircle class="animate-spin" />
		</div>
	{/if}
</div>

{#if $nextParam}
	{#key page}
		<Self session={$nextParam} page={page + 1} />
	{/key}
{/if}
