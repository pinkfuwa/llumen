<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ASTNode } from './lexer/parser';

	let { node, children }: { node: ASTNode; children: Snippet } = $props();

	// Extract URL from Link node structure
	// Lezer Link nodes typically have LinkMark children containing the URL
	function extractUrl(node: any): string | null {
		// Look for URL in children or text content
		// This is a simplified extraction - may need refinement based on actual structure
		const urlChild = node.children?.find((c: any) => c.type === 'URL');
		if (urlChild) return urlChild.text || '';

		// No URL found, this is a shortcut reference link like [abc]
		return null;
	}

	// Extract label text from shortcut reference link like [abc]
	function extractLabel(node: any): string {
		// The label is between the [ and ] marks
		// For node.text like "[abc]", extract "abc"
		const text = node.text || '';
		const match = text.match(/^\[([^\]]*)\]$/);
		return match ? match[1] : text;
	}

	const href = $derived(extractUrl(node));
	const isPlainText = $derived(href === null);
	const labelText = $derived(extractLabel(node));
</script>

{#if isPlainText}
	[{labelText}]
{:else}
	<span class="pb-1">
		<a
			href={href}
			target="_blank"
			rel="noopener noreferrer"
			class="rounded-xs border-b border-outline p-0.5 duration-150 hover:bg-primary hover:text-text-hover"
			>{@render children?.()}</a
		>
	</span>
{/if}
