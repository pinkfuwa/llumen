<script>
	let { depth, raw, text, children } = $props();
</script>

{#if depth === 1}
	<h1 class="mt-1 text-2xl font-bold">{@render children?.()}</h1>
{:else if depth === 2}
	<h2 class="mt-1 text-xl font-bold">{@render children?.()}</h2>
{:else if depth === 3}
	<h3 class="mt-1 text-lg font-bold">{@render children?.()}</h3>
{:else if depth === 4}
	<h4>{@render children?.()}</h4>
{:else if depth === 5}
	<h5>{@render children?.()}</h5>
{:else if depth === 6}
	<h6>{@render children?.()}</h6>
{:else}
	{raw}
{/if}
