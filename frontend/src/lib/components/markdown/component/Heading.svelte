<script lang="ts">
	import type { HeadingNode } from '../parser/types';
	import type { Snippet } from 'svelte';

	let { node, children }: { node: HeadingNode; children: Snippet } = $props();

	const classes = ['mt-4 text-2xl font-bold', 'mt-4 text-xl font-bold', 'mt-3 text-lg font-bold'];
	const classname = $derived.by(() => {
		const level = node.level - 1;
		if (level >= classes.length) return classes.at(-1)!;
		return classes[level];
	});
</script>

<h2 class={classname}>
	{@render children()}
</h2>
