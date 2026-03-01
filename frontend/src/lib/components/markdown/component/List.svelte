<script lang="ts">
	import type { OrderedListNode, UnorderedListNode } from '../parser/types';
	import { AstNodeType } from '../parser/types';
	import type { Snippet } from 'svelte';

	let {
		node,
		children
	}: { node: OrderedListNode | UnorderedListNode; children: Snippet } = $props();

	const isOrdered = $derived(node.type === AstNodeType.OrderedList);
	const startNumber = $derived(
		isOrdered ? (node as OrderedListNode).startNumber || 1 : undefined
	);

	const numberWidthRem = $derived(((startNumber || 1) + 10).toString().length * 0.7);
</script>

{#if isOrdered}
	<ol start={startNumber} class="list-decimal" style={`margin-left: ${numberWidthRem}rem`}>
		{@render children()}
	</ol>
{:else}
	<ul class="ml-4 list-disc">
		{@render children()}
	</ul>
{/if}
