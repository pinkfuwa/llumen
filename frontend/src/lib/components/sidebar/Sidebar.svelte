<script lang="ts">
	let {
		addition = false,
		currentRoom = undefined as undefined | number,
		collapsed = $bindable(false)
	} = $props();

	import { slide } from 'svelte/transition';
	import SettingBtn from './SettingBtn.svelte';
	import CollapseBtn from './CollapseBtn.svelte';
	import CollapseHeader from './CollapseHeader.svelte';
	import RoomPagination from '../room/RoomPagination.svelte';
</script>

{#if collapsed}
	<CollapseBtn onclick={() => (collapsed = false)} />
{:else}
	<header
		in:slide={{ duration: 180, axis: 'x' }}
		out:slide={{ duration: 180, axis: 'x' }}
		class="flex h-screen w-64 flex-col justify-between overflow-x-hidden border-r border-outline bg-background p-5 text-nowrap lg:w-80 xl:w-100"
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
	</header>
{/if}
