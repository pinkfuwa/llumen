<script lang="ts">
	import { copy } from '$lib/copy';
	import type { InlineCodeNode } from '../parser/types';

	let { node }: { node: InlineCodeNode } = $props();

	const content = $derived(node.content);

	const codeStyle =
		'my-0.5 rounded-md bg-muted px-2 py-0.5 font-mono break-all text-foreground duration-150 select-text hover:bg-interactive-hover';
</script>

<span class="mx-1 py-1">
	<button class={codeStyle} onclick={() => copy(content)}>{content}</button>
</span>
