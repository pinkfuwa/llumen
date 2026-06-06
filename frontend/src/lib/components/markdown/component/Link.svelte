<script lang="ts">
	import type { LinkNode } from '../vendor/types';
	import type { Snippet } from 'svelte';
	import RawMarker from './RawMarker.svelte';

	let { node, children }: { node: LinkNode; children: Snippet } = $props();

	const url = $derived(node.url);
	const title = $derived(node.title);

	const linkStyle =
		'rounded-xs border-b border-markdown-link p-0.5 text-markdown-link duration-150 hover:text-markdown-link-hover';
</script>

<span class="pb-1">
	<RawMarker text="[" />
	<a href={url} {title} target="_blank" rel="noopener noreferrer" class={linkStyle}>
		{@render children()}
	</a>
	<RawMarker text={`](${url})`} />
</span>
