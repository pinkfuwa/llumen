<script lang="ts">
	import { ArrowRight } from '@lucide/svelte';
	import { blur } from 'svelte/transition';
	import Button from '$lib/ui/Button.svelte';
	import { open } from './open.svelte';
</script>

<div
	class="fixed top-5 left-5 z-100 bg-card data-[state=open]:hidden"
	in:blur={{ duration: 180 }}
	data-state={open.val ? 'open' : 'close'}
>
	<Button
		onclick={() => {
			open.val = !open.val;
		}}
		class="bg-card p-3"
	>
		<ArrowRight />
	</Button>
</div>
