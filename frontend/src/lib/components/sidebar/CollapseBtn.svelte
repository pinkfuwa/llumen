<script lang="ts">
	import { ArrowRight } from '@lucide/svelte';
	import type { MouseEventHandler } from 'svelte/elements';
	import { blur, fly } from 'svelte/transition';
	import Button from '$lib/ui/Button.svelte';

	let { onclick = (() => {}) as MouseEventHandler<HTMLButtonElement> } = $props();
</script>

<div class="fixed top-5 left-5 z-100" in:blur={{ duration: 180 }}>
	<Button onclick={onclick} class="p-3 bg-sidebar-bg">
		<ArrowRight />
	</Button>
</div>
