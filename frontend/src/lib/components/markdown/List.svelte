<script>
	let { ordered, start, children } = $props();
</script>

{#if ordered}
	<ol class="ml-6 list-decimal" {start}>{@render children?.()}</ol>
{:else}
	<ul class="ml-6 list-disc">{@render children?.()}</ul>
{/if}
