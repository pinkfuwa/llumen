<script>
	let { header, align, children } = $props();
</script>

{#if header}
	<th {align} class="px-6 py-3 text-center text-xs font-medium tracking-wider uppercase"
		>{@render children?.()}</th
	>
{:else}
	<td {align} class="px-6 py-4 font-semibold whitespace-nowrap">{@render children?.()}</td>
{/if}
