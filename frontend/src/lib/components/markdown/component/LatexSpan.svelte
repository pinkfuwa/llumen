<script lang="ts">
	import type { LatexInlineNode } from '../parser/types';
	import LatexComponent from '../../latex/Latex.svelte';
	import { clickOnCopy } from './clickOnCopy.svelte';

	let { node }: { node: LatexInlineNode } = $props();

	const content = $derived(node.content);

	let element = $state<HTMLElement | null>(null);

	clickOnCopy(
		() => element,
		() => content
	);
</script>

<button
	class="inline-flex max-w-full items-center overflow-hidden rounded-md align-middle hover:bg-interactive-hover"
	bind:this={element}
>
	<span class="m-0.5 inline-block max-w-full overflow-x-auto overflow-y-hidden p-0.5 align-middle">
		<LatexComponent text={content} />
	</span>
</button>
