<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<strong class="text-markdown-bold">
	{@render children()}
</strong>
