<script>
	import { marked } from 'marked';
	import Parser from './Parser.svelte';

	// monochrome import shiki's performance
	let { source, monochrome = false } = $props();

	let tokens = new Promise((resolve) => {
		setTimeout(() => {
			const tokens = marked.lexer(source);
			resolve(tokens);
		}, 0);
	});
</script>

{#await tokens}
	{#each source.split('n') as line}
		<p>{line}</p>
	{/each}
{:then tokens}
	<Parser {tokens} {monochrome} />
{/await}
