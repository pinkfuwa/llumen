<script>
	/**
	 * @typedef {Object} Props
	 * @property {string} [href]
	 * @property {any} [title]
	 * @property {import('svelte').Snippet} [children]
	 */

	/** @type {Props} */
	let { href = '', title = undefined, children } = $props();
</script>

<span class="pb-1">
	<a {href} {title} target="_blank" class="border-b border-outline p-0.5 rounded-xs hover:bg-primary hover:text-text-hover duration-150"
		>{@render children?.()}</a
	>
</span>
