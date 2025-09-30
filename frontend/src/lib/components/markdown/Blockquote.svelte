<script>
	/**
	 * @typedef {Object} Props
	 * @property {import('svelte').Snippet} [children]
	 */

	/** @type {Props} */
	let { children } = $props();
</script>

<blockquote class="my-1 border-l-4 pl-3 hover:border-primary">{@render children?.()}</blockquote>
