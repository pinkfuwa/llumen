<script lang="ts">
	import type { TableCellNode } from '../parser/types';
	import type { Snippet } from 'svelte';

	let { node, children }: { node: TableCellNode; children: Snippet } = $props();

	const align = $derived(node.align);
	const alignClass = $derived(
		align === 'center' ? 'text-center' : align === 'right' ? 'text-right' : 'text-left'
	);
</script>

<td class="px-6 py-4 font-semibold {alignClass}">
	<div>
		{@render children?.()}
	</div>
</td>
