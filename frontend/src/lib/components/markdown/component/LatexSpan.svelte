<script lang="ts">
	import type { LatexInlineNode } from '../vendor/types';
	import LatexComponent from '../../latex/Latex.svelte';
	import RawMarker from './RawMarker.svelte';

	let { node }: { node: LatexInlineNode } = $props();

	const content = $derived(node.content);
</script>

<span class="inline-block max-w-full p-1 align-middle">
	<span class="inline-block max-w-full overflow-x-auto align-middle select-none">
		<LatexComponent text={content} />
	</span>
	<RawMarker text={`\\(${content}\\)`} />
</span>