<script lang="ts">
	let { node, monochrome = false, children } = $props();

	// Extract URL from Link node structure
	// Lezer Link nodes typically have LinkMark children containing the URL
	function extractUrl(node: any): string {
		if (node.url) return node.url;

		// Look for URL in children or text content
		// This is a simplified extraction - may need refinement based on actual structure
		const linkMarkChild = node.children?.find(
			(c: any) => c.type === 'LinkMark' || c.type === 'URL'
		);
		if (linkMarkChild) {
			return linkMarkChild.text || '';
		}

		return node.text || '#';
	}

	const href = extractUrl(node);
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
