<script lang="ts">
	import type { HeadingNode } from '../parser/types';
	import type { Snippet } from 'svelte';

	let { node, children }: { node: HeadingNode; children: Snippet } = $props();

	const sizes = ['mt-4 text-2xl', 'mt-4 text-xl', 'mt-3 text-lg'];
	const colors = [
		'text-markdown-heading1',
		'text-markdown-heading2',
		'text-markdown-heading3',
		'text-markdown-heading4'
	];
	const classname = $derived.by(() => {
		const level = node.level - 1;
		const size = level >= sizes.length ? sizes.at(-1)! : sizes[level];
		const color = level >= colors.length ? colors.at(-1)! : colors[level];
		return `${size} font-bold ${color}`;
	});
</script>

<h2 class={classname}>
	{@render children()}
</h2>
