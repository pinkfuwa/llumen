<script lang="ts">
	import type { LatexInlineNode } from '../parser/types';
	import LatexComponent from '../../latex/Latex.svelte';

	let { node }: { node: LatexInlineNode } = $props();

	const content = $derived(node.content);
</script>

<span class="inline-block p-1 align-middle">
	<span class="inline-block max-w-full overflow-x-auto align-middle text-lg">
		<LatexComponent text={content} />
	</span>
</span>
