<script lang="ts">
	import { copy } from '$lib/copy';
	import type { InlineCodeNode } from '../parser/types';

	let { node }: { node: InlineCodeNode } = $props();

	const content = $derived(node.content);
</script>

<span class="mx-1 py-1">
	<button
		class="my-0.5 cursor-pointer rounded-md bg-secondary px-2 py-0.5 font-mono break-all text-text duration-150 hover:bg-primary hover:text-text-hover"
		onclick={() => copy(content)}>{content}</button
	>
</span>
