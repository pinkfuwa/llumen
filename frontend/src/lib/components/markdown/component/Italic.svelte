<script lang="ts">
	import type { Snippet } from 'svelte';
	import RawMarker from './RawMarker.svelte';

	let { children }: { children: Snippet } = $props();
</script>

<em class="text-markdown-italic">
	<RawMarker text="*" />
	{@render children()}
	<RawMarker text="*" />
</em>