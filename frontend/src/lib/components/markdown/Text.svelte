<script lang="ts">
	import type { ASTNode } from './lexer/parser';

	let { node }: { node: ASTNode } = $props();
</script>

<span>{node.text}</span>
