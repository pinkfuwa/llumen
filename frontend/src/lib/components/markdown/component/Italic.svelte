<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<em class="text-markdown-italic">
	{@render children()}
</em>
