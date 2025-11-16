<script lang="ts">
	import { CornerDownLeft } from '@lucide/svelte';
	import type { MouseEventHandler } from 'svelte/elements';
	let { onclick = (() => {}) as MouseEventHandler<HTMLButtonElement>, disabled = false } = $props();
</script>

<button
	class="-mr-2 rounded-md p-2 duration-150 hover:bg-primary hover:text-text-hover not-disabled:hover:bg-primary not-disabled:hover:text-text-hover focus:ring-4 focus:ring-outline focus:outline-none disabled:cursor-not-allowed disabled:opacity-60"
	{disabled}
	{onclick}
	aria-label="send"
>
	<CornerDownLeft class="inline-block" />
</button>
