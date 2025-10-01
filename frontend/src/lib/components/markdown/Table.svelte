<script>
	/**
	 * @typedef {Object} Props
	 * @property {import('svelte').Snippet} [children]
	 */

	/** @type {Props} */
	let { children } = $props();
</script>

<div class="flex justify-center">
	<div class="relative overflow-x-auto rounded-md border border-outline">
		<table
			class="divide-y divide-gray-200 bg-component-bg whitespace-nowrap lg:whitespace-break-spaces"
		>
			{@render children?.()}
		</table>
	</div>
</div>
