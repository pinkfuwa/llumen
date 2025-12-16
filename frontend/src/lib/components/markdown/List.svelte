<script lang="ts">
	import type { Token, OrderedListToken } from './lexer';
	import { TokenType } from './lexer';
	import type { Snippet } from 'svelte';

	let { token, source, children }: { token: Token; source: string; children: Snippet } = $props();

	const isOrdered = $derived(token.type === TokenType.OrderedList);
	const startNumber = $derived(
		isOrdered ? (token as OrderedListToken).startNumber || 1 : undefined
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
