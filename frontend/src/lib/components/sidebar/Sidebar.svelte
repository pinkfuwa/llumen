<script lang="ts">
	let { addition = false, open = $bindable(false) } = $props();

	import CollapseHeader from './CollapseHeader.svelte';
	import RoomPagination from '../room/RoomPagination.svelte';
	import Setting from '../setting/Setting.svelte';
	import { createSwipeGesture } from './gesture';

	let sidebarElement = $state<HTMLElement | null>(null);

	$effect(() => {
		if (!sidebarElement) return;

		const cleanup = createSwipeGesture(sidebarElement, {
			threshold: 50,
			velocity: 0.3,
			onSwipe: (direction) => {
				if (direction === 'left' && open) {
					open = false;
				} else if (direction === 'right' && !open) {
					open = true;
				}
			}
		});

		return cleanup;
	});
</script>

<header
	bind:this={sidebarElement}
	class="flex h-screen w-screen flex-col justify-between border-outline bg-sidebar-bg p-5 transition-all data-[state=close]:-ml-[100vw] md:w-[min(calc(160px+20rem),33vw)] md:border-r md:data-[state=close]:-ml-[min(calc(160px+20rem),33vw)]"
	data-state={open ? 'open' : 'close'}
>
	<div class="mb-4 shrink-0 border-b border-outline pb-1">
		<CollapseHeader onclick={() => (open = !open)} />
	</div>
	<div class="nobar min-w-0 grow overflow-y-auto">
		<RoomPagination {addition} />
	</div>
	<div class="mt-4 shrink-0 border-t border-outline pt-4">
		<Setting />
	</div>
</header>
