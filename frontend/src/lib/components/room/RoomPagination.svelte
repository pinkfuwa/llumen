<script lang="ts">
	let { addition = false, currentRoom = undefined as undefined | number } = $props();
	import ChatroomBtn from './ChatroomBtn.svelte';
	import { LoaderCircle, Plus } from '@lucide/svelte';
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
	<ChatroomBtn name="test" id={1} />
	<ChatroomBtn name="test" id={2} />
	<li>
		class="mt-1 flex justify-center">
		<LoaderCircle class="animate-spin" />
	</li>
</ul>
