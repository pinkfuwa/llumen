<script lang="ts">
	let {
		addition = false,
		currentRoom = undefined as undefined | string,
		collapsed = $bindable(false)
	} = $props();

	import { slide } from 'svelte/transition';
	import SettingBtn from './buttons/SettingBtn.svelte';
	import CollapseBtn from './buttons/CollapseBtn.svelte';
	import CollapseHeader from './CollapseHeader.svelte';
	import RoomPagination from './room/RoomPagination.svelte';
</script>

{#if collapsed}
	<CollapseBtn onclick={() => (collapsed = false)} />
{:else}
	<div
		in:slide={{ duration: 180, axis: 'x' }}
		out:slide={{ duration: 180, axis: 'x' }}
		class="flex h-screen w-64 flex-col justify-between border-r border-outline bg-background p-5 lg:w-80 xl:w-100"
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
