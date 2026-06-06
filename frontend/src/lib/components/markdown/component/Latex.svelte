<script lang="ts">
	import type { LatexBlockNode } from '../vendor/types';
	import LatexComponent from '../../latex/Latex.svelte';
	import RawMarker from './RawMarker.svelte';

	let { node }: { node: LatexBlockNode } = $props();

	const content = $derived(node.content);
</script>

<div class="flex justify-center">
	<div class="max-w-full overflow-x-auto p-1.5 text-center">
		<LatexComponent text={content} displayMode />
	</div>
	<RawMarker text={`\[${content}\]`} block />
</div>
