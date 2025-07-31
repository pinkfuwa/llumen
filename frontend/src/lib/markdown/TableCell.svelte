<script>
	let { header, align, children } = $props();
</script>

{#if header}
	<th
		{align}
		class="px-6 py-3 text-right text-xs font-medium tracking-wider text-gray-700 uppercase"
		>{@render children?.()}</th
	>
{:else}
	<td {align} class="px-6 py-4 font-semibold whitespace-nowrap text-gray-900"
		>{@render children?.()}</td
	>
{/if}
