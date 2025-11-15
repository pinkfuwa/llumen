<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ASTNode } from './lexer/parser';

	let { node, children }: { node: ASTNode; children: Snippet } = $props();

	// Extract URL from Link node structure
	// Lezer Link nodes typically have LinkMark children containing the URL
	function extractUrl(node: any): string {
		// Look for URL in children or text content
		// This is a simplified extraction - may need refinement based on actual structure
		const urlChild = node.children?.find((c: any) => c.type === 'URL');
		if (urlChild) return urlChild.text || '';

		return node.text || '#';
	}

	const href = $derived(extractUrl(node));
</script>

<span class="pb-1">
	<a
		{href}
		target="_blank"
		rel="noopener noreferrer"
		class="rounded-xs border-b border-outline p-0.5 duration-150 hover:bg-primary hover:text-text-hover"
		>{@render children?.()}</a
	>
</span>
