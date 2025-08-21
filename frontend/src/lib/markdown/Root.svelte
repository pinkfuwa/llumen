<script>
	import { marked } from 'marked';
	import Parser from './Parser.svelte';

	let { source, monochrome = false } = $props();

	let tokens = marked.lexer(source);
</script>

<Parser {tokens} {monochrome} />
