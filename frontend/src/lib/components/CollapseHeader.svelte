<script lang="ts">
	import { ArrowLeft, Bot } from '@lucide/svelte';
	import type { MouseEventHandler } from 'svelte/elements';

	let { onclick = (() => {}) as MouseEventHandler<HTMLButtonElement> } = $props();
</script>

<div class="flex items-center justify-between">
	<div class="flex items-center text-lg font-semibold">
		<Bot class="mx-2 h-6 w-6" /> <span>llumen</span>
	</div>
	<button class="rounded-lg p-3 text-dark hover:bg-hover" {onclick}>
		<ArrowLeft />
	</button>
</div>
