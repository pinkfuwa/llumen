<script lang="ts">
	import { copy } from '$lib/copy.svelte';
	import type { InlineCodeNode } from '../parser/types';

	let { node }: { node: InlineCodeNode } = $props();

	const content = $derived(node.content);

	const codeStyle =
		'my-0.5 rounded-md bg-markdown-inline-code-bg px-2 py-0.5 font-mono break-all text-markdown-inline-code duration-150 select-text hover:bg-interactive-hover';
</script>

<span class="mx-1 py-1">
	<button class={codeStyle} onclick={() => copy(content)}>{content}</button>
</span>
