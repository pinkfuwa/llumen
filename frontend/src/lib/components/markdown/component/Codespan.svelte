<script lang="ts">
	import type { InlineCodeNode } from '../parser/types';
	import RawMarker from './RawMarker.svelte';

	let { node }: { node: InlineCodeNode } = $props();

	const content = $derived(node.content);
</script>

<span class="mx-1 py-1">
	<span
		class="my-0.5 rounded-md bg-markdown-inline-code-bg px-2 py-0.5 font-mono break-all text-markdown-inline-code duration-150 select-text hover:bg-interactive-hover"
		>{content}</span
	>
	<RawMarker text={`\`${content}\``} />
</span>
