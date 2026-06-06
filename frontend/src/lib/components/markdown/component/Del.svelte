<script lang="ts">
	import type { Snippet } from 'svelte';
	import RawMarker from './RawMarker.svelte';

	let { children }: { children: Snippet } = $props();
</script>

<del class="text-markdown-strikethrough">
	<RawMarker text="~~" />
	{@render children()}
	<RawMarker text="~~" />
</del>
