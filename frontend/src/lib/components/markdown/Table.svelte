<script lang="ts">
	import Parser from './Parser.svelte';
	import type { ASTNode } from './lexer/parser';

	let { node }: { node: ASTNode } = $props();
</script>

<div class="my-1 flex justify-center">
	<div class="relative overflow-x-auto rounded-md border border-outline">
		<table
			class="divide-y divide-gray-200 bg-component-bg whitespace-nowrap lg:whitespace-break-spaces"
		>
			{#if node.children && node.children.length}
				{#each node.children as child}
					<Parser ast={child} />
				{/each}
			{/if}
		</table>
	</div>
</div>
