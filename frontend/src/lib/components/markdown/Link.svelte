<script lang="ts">
	import type { ASTNode } from './lexer/parser';

	let { node }: { node: ASTNode } = $props();
	let url = $derived(node.children.find((x) => x.type == 'URL'));

	let childrenText = $derived.by(() => {
		let marks = node.children.filter((x) => x.type == 'LinkMark');
		if (marks.length <= 2) return;
		let [first, second] = marks;

		return node.text.slice(first.to - 1, second.from - 1);
	});
</script>

{#if url == undefined || childrenText == undefined}
	<span>{node.text}</span>
{:else}
	<span class="pb-1">
		<a
			href={url.text}
			target="_blank"
			rel="noopener noreferrer"
			class="rounded-xs border-b border-outline p-0.5 duration-150 hover:bg-primary hover:text-text-hover"
		>
			{childrenText}
		</a>
	</span>
{/if}
