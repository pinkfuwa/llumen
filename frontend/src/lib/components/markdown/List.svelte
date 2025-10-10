<script>
	let { ordered, start, children } = $props();

	let decimalDigit = $derived(start.toString().length);

	let marginLeft = $derived(decimalDigit <= 2 ? 24 : decimalDigit * 12);
</script>

{#if ordered}
	<ol class="list-decimal" {start} style={`margin-left: ${marginLeft}px`}>
		{@render children?.()}
	</ol>
{:else}
	<ul class="ml-6 list-disc">{@render children?.()}</ul>
{/if}
