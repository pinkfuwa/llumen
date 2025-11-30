<script lang="ts">
	import { SquareCheck } from '@lucide/svelte';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<div class="flex flex-row items-center">
	<SquareCheck class="mr-2 min-h-8 min-w-8 p-1" />
	<span>
		{@render children?.()}
	</span>
</div>
