<script lang="ts">
	import Parser from './Parser.svelte';
	import { lex, getCachedLex } from './worker';

	// monochrome import shiki's performance
	let { source, monochrome = false } = $props();

	let cached = $derived(getCachedLex(source));
</script>

{#if cached == null}
	{#await lex(source, true)}
		{#each source.split('\n') as line}
			<p>{line}</p>
		{/each}
	{:then tokens}
		{#key source}
			<Parser {tokens} {monochrome} />
		{/key}
	{/await}
{:else}
	{#key cached}
		<Parser tokens={cached} {monochrome} />
	{/key}
{/if}
