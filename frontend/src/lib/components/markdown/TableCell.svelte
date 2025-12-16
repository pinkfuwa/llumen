<script lang="ts">
	import type { TableCellToken } from './lexer';
	import type { Snippet } from 'svelte';

	let { token, children }: { token: TableCellToken; children: Snippet } = $props();

	const align = $derived(token.align);
	const alignClass = $derived(
		align === 'center' ? 'text-center' : align === 'right' ? 'text-right' : 'text-left'
	);
</script>

<td class="px-6 py-4 font-semibold {alignClass}">
	<div>
		{@render children?.()}
	</div>
</td>
