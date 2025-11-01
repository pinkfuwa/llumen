<script lang="ts">
	let { node, monochrome = false, children } = $props();

	// Determine if this is an ordered or unordered list
	const isOrdered = node.type === 'OrderedList';

	// todo: get actual numbering
	let start = 100;
	let decimalDigit = $derived(start.toString().length);
	let marginLeft = $derived(decimalDigit <= 2 ? 1.5 : decimalDigit * 0.75);
</script>

{#if isOrdered}
	<ol class="list-decimal" {start} style={`margin-left: ${marginLeft}rem`}>
		{@render children?.()}
	</ol>
{:else}
	<ul class="ml-6 list-disc">{@render children?.()}</ul>
{/if}
