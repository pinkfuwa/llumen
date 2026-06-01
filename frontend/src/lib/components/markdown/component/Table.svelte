<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	const wrapperStyle = 'relative my-1 overflow-x-auto rounded-md border border-border';
	const tableStyle = 'divide-y divide-border bg-card whitespace-nowrap lg:whitespace-break-spaces';
</script>

<div class="flex justify-center">
	<div class={wrapperStyle}>
		<table class={tableStyle}>
			{@render children()}
		</table>
	</div>
</div>
