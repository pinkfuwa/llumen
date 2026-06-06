<script lang="ts">
	import type { LatexInlineNode } from '../vendor/types';
	import LatexComponent from '../../latex/Latex.svelte';
	import RawMarker from './RawMarker.svelte';

	let { node }: { node: LatexInlineNode } = $props();

	const content = $derived(node.content);
</script>

<span class="inline-flex max-w-full items-center overflow-hidden align-middle">
	<span
		class="m-[0.125rem] inline-block max-w-full overflow-x-auto overflow-y-hidden p-[0.125rem] align-middle select-none"
	>
		<LatexComponent text={content} />
	</span>
	<RawMarker text={`\\(${content}\\)`} />
</span>
