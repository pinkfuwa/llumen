<script>
	import { Lexer } from 'marked';
	import Parser from './Parser.svelte';

	const lexer = new Lexer();

	let { source } = $props();

	let tokens = lexer.lex(source);
</script>

<Parser {tokens} />
