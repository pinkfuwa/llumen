<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<div class="flex justify-center">
	<div class="relative my-1 overflow-x-auto rounded-md border border-border">
		<table class="divide-y divide-border bg-card whitespace-nowrap lg:whitespace-break-spaces">
			{@render children()}
		</table>
	</div>
</div>
