<script lang="ts">
	import { ArrowRight } from '@lucide/svelte';
	import type { MouseEventHandler } from 'svelte/elements';

	let { onclick = (() => {}) as MouseEventHandler<HTMLButtonElement> } = $props();
</script>

<div class="fixed top-5 left-5">
	<button
		class="rounded-lg border border-outline bg-white p-2 text-dark shadow hover:bg-hover"
		{onclick}
	>
		<ArrowRight />
	</button>
</div>
