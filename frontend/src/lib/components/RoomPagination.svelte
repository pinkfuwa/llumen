<script lang="ts">
	let { addition = false } = $props();

	import { Plus } from '@lucide/svelte';
	import ForwardPage from './room/ForwardPage.svelte';
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
	<!-- {#each $roomlist.data as room}
		<a href="/chat/{encodeURIComponent(room.id)}">
			<ChatroomBtn name={room.title} />
		</a>
	{/each} -->
	<ForwardPage />
</ul>
