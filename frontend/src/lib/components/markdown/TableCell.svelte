<script>
	let { header, align, children } = $props();
</script>

{#if header}
	<th {align} class="px-6 py-3 text-center text-xs tracking-wider uppercase"
		>{@render children?.()}</th
	>
{:else}
	<td {align} class="px-6 py-4 font-semibold">
		<div>
			{@render children?.()}
		</div>
	</td>
{/if}
