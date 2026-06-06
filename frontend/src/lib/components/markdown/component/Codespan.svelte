<script lang="ts">
	import type { InlineCodeNode } from '../vendor/types';
	import RawMarker from './RawMarker.svelte';

	let { node }: { node: InlineCodeNode } = $props();

	const content = $derived(node.content);

	const codeStyle =
		'my-0.5 rounded-md bg-markdown-inline-code-bg px-2 py-0.5 font-mono break-all text-markdown-inline-code duration-150 select-text hover:bg-interactive-hover';
</script>

<span class="mx-1 py-1">
	<span class={codeStyle}>{content}</span>
	<RawMarker text={`\`${content}\``} />
</span>
