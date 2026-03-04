<script lang="ts">
	import type { LatexBlockNode } from '../parser/types';
	import LatexComponent from '../../latex/Latex.svelte';
	import { copy } from '$lib/copy';

	let { node }: { node: LatexBlockNode } = $props();

	const content = $derived(node.content);

	function copyToClipboard() {
		copy(content);
	}
</script>

<div class="flex justify-center">
	<button
		type="button"
		class="max-w-full cursor-pointer overflow-x-auto p-1.5 text-center text-lg hover:opacity-80"
		onclick={copyToClipboard}
		title="Click to copy LaTeX"
	>
		<LatexComponent text={content} displayMode />
	</button>
</div>
