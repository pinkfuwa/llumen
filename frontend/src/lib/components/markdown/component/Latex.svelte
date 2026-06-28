<script lang="ts">
	import type { LatexBlockNode } from '../parser/types';
	import LatexComponent from '../../latex/Latex.svelte';
	import { clickOnCopy } from './clickOnCopy.svelte';

	let { node }: { node: LatexBlockNode } = $props();

	const content = $derived(node.content);
	let element = $state<HTMLElement | null>(null);

	clickOnCopy(
		() => element,
		() => content
	);
</script>

<button class="flex max-w-full justify-center hover:bg-interactive-hover" bind:this={element}>
	<div class="max-w-full overflow-x-auto p-1.5 text-center">
		<LatexComponent text={content} displayMode />
	</div>
</button>
