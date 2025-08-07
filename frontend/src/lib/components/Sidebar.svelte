<script lang="ts">
	let { addition = false, currentRoom = undefined as undefined | string } = $props();

	import { slide } from 'svelte/transition';
	import SettingBtn from './buttons/SettingBtn.svelte';
	import CollapseBtn from './buttons/CollapseBtn.svelte';
	import CollapseHeader from './CollapseHeader.svelte';
	import RoomPagination from './room/RoomPagination.svelte';

	let collapsed = $state(false);
</script>

{#if collapsed}
	<CollapseBtn onclick={() => (collapsed = false)} />
{:else}
	<div
		in:slide={{ duration: 180, axis: 'x' }}
		out:slide={{ duration: 180, axis: 'x' }}
		class="flex h-screen w-full flex-col justify-between border-r border-outline bg-background p-5"
	>
		<div>
			<div class="mb-4 border-b border-outline pb-1">
				<CollapseHeader onclick={() => (collapsed = true)} />
			</div>

			<RoomPagination {addition} {currentRoom} />
		</div>
		<div class="mt-4 border-t border-outline pt-4">
			<SettingBtn />
		</div>
	</div>
{/if}
