<script lang="ts">
	let { depth, raw } = $props<{ depth: number; raw: string }>();
	let text = $derived(raw.trim().replace(/^#+/gm, ''));
</script>

{#if depth === 1}
	<h1 class="mt-1 text-2xl font-bold">{text}</h1>
{:else if depth === 2}
	<h2 class="mt-1 text-xl font-bold">{text}</h2>
{:else if depth === 3}
	<h3 class="mt-1 text-lg font-bold">{text}</h3>
{:else if depth === 4}
	<h4>{text}</h4>
{:else if depth === 5}
	<h5>{text}</h5>
{:else if depth === 6}
	<h6>{text}</h6>
{:else}
	{raw}
{/if}
