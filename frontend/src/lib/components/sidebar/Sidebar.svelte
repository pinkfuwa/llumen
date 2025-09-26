<script lang="ts">
	let {
		addition = false,
		currentRoom = undefined as undefined | number,
		open = $bindable(false)
	} = $props();

	import CollapseHeader from './CollapseHeader.svelte';
	import RoomPagination from '../room/RoomPagination.svelte';
	import Setting from '../setting/Setting.svelte';
</script>

<header
	class="flex h-screen w-screen flex-col justify-between border-r border-outline bg-sidebar-bg p-5 slide-in-from-left slide-out-to-left data-[state=close]:hidden data-[state=close]:animate-out data-[state=open]:animate-in md:w-[min(400px,33vw)]"
	data-state={open ? 'open' : 'close'}
>
	<div>
		<div class="mb-4 border-b border-outline pb-1">
			<CollapseHeader onclick={() => (open = !open)} />
		</div>

		<RoomPagination {addition} {currentRoom} />
	</div>
	<div class="mt-4 border-t border-outline pt-4">
		<Setting />
	</div>
</header>
