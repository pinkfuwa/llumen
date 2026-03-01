<script lang="ts">
	import type { LatexBlockNode } from '../parser/types';
	import LatexComponent from '../../latex/Latex.svelte';

	let { node }: { node: LatexBlockNode } = $props();

	const content = $derived(node.content);
</script>

<div class="flex justify-center">
	<div class="max-w-full overflow-x-auto p-1.5 text-lg">
		<LatexComponent text={content} displayMode />
	</div>
</div>
