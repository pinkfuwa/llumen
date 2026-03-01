<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<blockquote class="my-1 space-y-1 border-l-4 pl-3 hover:border-primary">
	{@render children?.()}
</blockquote>
