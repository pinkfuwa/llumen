<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { ASTNode } from './lexer/parser';

	// *del* are rendered as delete in standard markdown
	// But standard isn't the behavior user expected

	let { children }: { children: Snippet; node: ASTNode } = $props();
</script>

<del>
	{@render children?.()}
</del>
