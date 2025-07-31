<script>
	/**
	 * @typedef {Object} Props
	 * @property {import('svelte').Snippet} [children]
	 */

	/** @type {Props} */
	let { children } = $props();
</script>

<table class="divide-y divide-gray-200 rounded-md border border-outline bg-white">
	{@render children?.()}
</table>
