<script>
	import { Lexer } from 'marked';
	import Parser from './Parser.svelte';

	let { source, monochrome = false } = $props();

	const lexer = new Lexer();

	let tokens = lexer.lex(source);
</script>

<Parser {tokens} {monochrome} />
