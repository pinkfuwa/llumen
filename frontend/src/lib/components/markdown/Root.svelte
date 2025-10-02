<script lang="ts">
	import Parser from './Parser.svelte';
	import { lex } from './worker';

	// monochrome import shiki's performance
	let { source, monochrome = false } = $props();

	let tokens = $derived.by(() => lex(source, true));
</script>

{#await tokens}
	{#each source.split('\n') as line}
		<p>{line}</p>
	{/each}
{:then tokens}
	{#key source}
		<Parser {tokens} {monochrome} />
	{/key}
{/await}
