<script lang="ts">
	import type { LinkToken } from './lexer';
	import type { Snippet } from 'svelte';

	let { token, children }: { token: LinkToken; children: Snippet } = $props();

	const linkToken = $derived(token);
	const url = $derived(linkToken.url);
	const title = $derived(linkToken.title);
</script>

<span class="pb-1">
	<a
		href={url}
		{title}
		target="_blank"
		rel="noopener noreferrer"
		class="rounded-xs border-b border-outline p-0.5 duration-150 hover:bg-primary hover:text-text-hover"
	>
		{@render children()}
	</a>
</span>
