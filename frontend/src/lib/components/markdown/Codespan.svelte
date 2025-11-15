<script lang="ts">
	import { copy } from '$lib/copy';

	let { node }: { node: ASTNode } = $props();

	// Extract code text, excluding the backtick markers
	function extractCodeText(node: ASTNode): string {
		// InlineCode nodes may have CodeMark children (the backticks)
		// We want the actual text content between them
		const codeTextChild = node.children?.find(
			(c) => c.type === 'CodeText' || c.type === 'InlineCode'
		);
		if (codeTextChild) {
			return codeTextChild.text!;
		}
		return node.text || '';
	}

	const text = $derived(extractCodeText(node).replace(/`/g, ''));
</script>

<span class="mx-1 py-1">
	<button
		class="my-0.5 cursor-pointer rounded-md bg-secondary px-2 py-0.5 font-mono break-all text-text duration-150 hover:bg-primary hover:text-text-hover"
		onclick={() => copy(text)}>{text}</button
	>
</span>
