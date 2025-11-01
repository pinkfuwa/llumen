<script lang="ts">
	let { node, monochrome = false } = $props();

	// Extract image attributes from Lezer node structure
	function extractImageAttributes(node: any): { src: string; alt: string; title?: string } {
		let src = '';
		let alt = '';
		let title: string | undefined;

		// If attributes are already extracted
		if (node.src) {
			return { src: node.src, alt: node.alt || '', title: node.title };
		}

		// Look for image components in children
		// Lezer Image nodes typically have LinkMark children containing URL
		if (node.children) {
			for (const child of node.children) {
				if (child.type === 'LinkMark' || child.type === 'URL') {
					src = child.text || '';
				} else if (child.type === 'LinkLabel') {
					// Link label contains the alt text
					alt = child.text || '';
				} else if (child.type === 'LinkTitle') {
					// Optional title attribute
					title = child.text || undefined;
				}
			}
		}

		// Fallback to node text if no URL found
		if (!src) {
			src = node.text || '';
		}

		return { src, alt, title };
	}

	const { src, alt, title } = extractImageAttributes(node);
</script>

<img {src} {alt} {title} style="max-width: 100%;" class:monochrome />
