<script lang="ts">
	import type { InlineCodeNode } from '../parser/types';
	import { clickOnCopy } from './clickOnCopy.svelte';

	let { node }: { node: InlineCodeNode } = $props();

	const content = $derived(node.content);

	let element = $state<HTMLElement | null>(null);

	clickOnCopy(
		() => element,
		() => content
	);
</script>

<span class="mx-1 py-1">
	<button
		class="my-0.5 rounded-md bg-markdown-inline-code-bg px-2 py-0.5 font-mono break-all text-markdown-inline-code duration-150 select-text hover:bg-interactive-hover"
		bind:this={element}>{content}</button
	>
</span>
