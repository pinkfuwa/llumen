<script lang="ts">
	import type { LatexInlineNode } from '../parser/types';
	import LatexComponent from '../../latex/Latex.svelte';
	import { copy } from '$lib/copy';

	let { node }: { node: LatexInlineNode } = $props();

	const content = $derived(node.content);

	function copyToClipboard() {
		copy(content);
	}
</script>

<span class="inline-block p-1 align-middle">
	<button
		type="button"
		class="inline-block max-w-full cursor-pointer overflow-x-auto align-middle select-none hover:opacity-80"
		onclick={copyToClipboard}
		title="Click to copy LaTeX"
	>
		<LatexComponent text={content} />
	</button>
	<span class="block max-h-0 max-w-0 overflow-hidden text-ellipsis">
		\({content}\)
	</span>
</span>
