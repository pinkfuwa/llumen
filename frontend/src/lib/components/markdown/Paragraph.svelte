<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<p class="space-y-2 align-middle">
	{@render children()}
</p>
