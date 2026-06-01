<script lang="ts">
	import type { LinkNode } from '../parser/types';
	import type { Snippet } from 'svelte';

	let { node, children }: { node: LinkNode; children: Snippet } = $props();

	const url = $derived(node.url);
	const title = $derived(node.title);
</script>

<span class="pb-1">
	<a
		href={url}
		{title}
		target="_blank"
		rel="noopener noreferrer"
		class="hover:text-inverse rounded-xs border-b border-outline p-0.5 duration-150 hover:bg-accent"
	>
		{@render children()}
	</a>
</span>
