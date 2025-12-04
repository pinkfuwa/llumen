<script lang="ts">
	import { Square } from '@lucide/svelte';
	import type { MouseEventHandler } from 'svelte/elements';
	let { onclick = (() => {}) as MouseEventHandler<HTMLButtonElement> } = $props();
</script>

<button class="-mr-2 rounded-md p-2 duration-150 hover:bg-primary hover:text-text-hover" {onclick}>
	<Square class="inline-block h-5 w-5" />
</button>
