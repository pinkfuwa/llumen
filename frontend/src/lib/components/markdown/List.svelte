<script lang="ts">
	import type { ASTNode } from './lexer/parser';
	import type { Snippet } from 'svelte';

	let { node, children }: { node: ASTNode; children: Snippet } = $props();

	const isOrdered = node.type === 'OrderedList';
</script>

{#if isOrdered}
	<ol class="list-decimal" style={`margin-left: 2.5rem`}>
		{@render children?.()}
	</ol>
{:else}
	<ul class="ml-6 list-disc">{@render children?.()}</ul>
{/if}
