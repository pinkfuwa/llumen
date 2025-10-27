<script>
	let { ordered, start, children } = $props();

	let decimalDigit = $derived(start.toString().length);

	let marginLeft = $derived(decimalDigit <= 2 ? 1.5 : decimalDigit * 0.75);
</script>

{#if ordered}
	<ol class="list-decimal" {start} style={`margin-left: ${marginLeft}rem`}>
		{@render children?.()}
	</ol>
{:else}
	<ul class="ml-6 list-disc">{@render children?.()}</ul>
{/if}
