<script>
	import { lexer } from './cache';
	import Parser from './Parser.svelte';

	let { source } = $props();

	let tokens = lexer.run(source);
</script>

<Parser {tokens} />
