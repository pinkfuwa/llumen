<script>
	let { ordered, start, children } = $props();
</script>

{#if ordered}
	<ol class="ml-6 list-decimal break-all" {start}>{@render children?.()}</ol>
{:else}
	<ul class="ml-6 list-disc break-all">{@render children?.()}</ul>
{/if}
