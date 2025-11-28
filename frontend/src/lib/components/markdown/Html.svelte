<script lang="ts">
	import type { ASTNode } from './lexer/parser';

	let { node }: { node: ASTNode } = $props();
</script>

{#if node.text == undefined}{:else if /^<br\s*\/?>$/i.test(node.text.trim())}
	<br />
{:else}
	{node.text}
{/if}
