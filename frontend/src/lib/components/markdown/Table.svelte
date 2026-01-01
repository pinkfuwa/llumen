<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<div class="flex touch-none justify-center">
	<div class="relative my-1 overflow-x-auto rounded-md border border-outline">
		<table
			class="divide-y divide-gray-200 bg-component-bg whitespace-nowrap lg:whitespace-break-spaces"
		>
			{@render children()}
		</table>
	</div>
</div>
