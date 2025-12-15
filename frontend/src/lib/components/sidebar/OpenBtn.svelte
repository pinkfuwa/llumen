<script lang="ts">
	import { ArrowRight } from '@lucide/svelte';
	import { blur } from 'svelte/transition';
	import Button from '$lib/ui/Button.svelte';

	let { class: className = '', open = $bindable(false) } = $props();
</script>

<div
	class="fixed top-5 left-5 z-100 data-[state=open]:hidden"
	in:blur={{ duration: 180 }}
	data-state={open ? 'open' : 'close'}
>
	<Button
		onclick={() => {
			open = !open;
		}}
		class={`bg-sidebar-bg p-3 ${className}`}
	>
		<ArrowRight />
	</Button>
</div>
