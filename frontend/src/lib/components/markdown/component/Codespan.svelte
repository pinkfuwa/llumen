<script lang="ts">
	import { copy } from '$lib/copy';
	import type { InlineCodeNode } from '../parser/types';

	let { node }: { node: InlineCodeNode } = $props();

	const content = $derived(node.content);
</script>

<span class="mx-1 py-1">
	<button
		class="text-primary my-0.5 rounded-md bg-accent-soft px-2 py-0.5 font-mono break-all duration-150 select-text hover:bg-accent hover:text-inverse"
		onclick={() => copy(content)}>{content}</button
	>
</span>
