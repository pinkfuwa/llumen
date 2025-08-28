<script lang="ts">
	import { Square } from '@lucide/svelte';
	import type { MouseEventHandler } from 'svelte/elements';
	let { onclick = (() => {}) as MouseEventHandler<HTMLButtonElement> } = $props();
</script>

<button class="mr-[-5px] rounded-md p-[6px] hover:bg-hover" {onclick}>
	<Square class="inline-block h-5 w-5" />
</button>
