<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<del class="text-markdown-strikethrough">
	{@render children()}
</del>
