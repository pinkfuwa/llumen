<script>
	import { marked } from 'marked';

	import Parser from './Parser.svelte';

	let { source } = $props();

	const lexer = new marked.Lexer();
	let tokens = lexer.lex(source);
</script>

<Parser {tokens} />
