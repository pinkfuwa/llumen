<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<blockquote class="my-1 space-y-1 border-l-4 border-markdown-blockquote-border pl-3 text-markdown-blockquote">
	{@render children?.()}
</blockquote>
