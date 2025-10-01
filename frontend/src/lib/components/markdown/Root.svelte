<script lang="ts">
	import { marked } from 'marked';
	import Parser from './Parser.svelte';

	// monochrome import shiki's performance
	let { source, monochrome = false } = $props();

	function renderToken(source: string) {
		return new Promise((resolve) => {
			setTimeout(() => resolve(marked.lexer(source)), 0);
		});
	}

	let tokens = $derived.by(() => renderToken(source));
</script>

{#await tokens}
	<!-- this is intentional, source.split('\n') is almost as resource consuming as lexer -->
	{source}
{:then tokens}
	{#key source}
		<Parser {tokens} {monochrome} />
	{/key}
{/await}
