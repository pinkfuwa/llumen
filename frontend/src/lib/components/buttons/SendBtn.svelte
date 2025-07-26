<script lang="ts">
	import { CornerDownLeft } from '@lucide/svelte';
	import type { MouseEventHandler } from 'svelte/elements';
	let { onclick = (() => {}) as MouseEventHandler<HTMLButtonElement> } = $props();
</script>

<button class="mr-[-5px] rounded-md p-1 hover:bg-hover" {onclick}>
	<CornerDownLeft class="inline-block" />
</button>
