<script lang="ts">
	import type { Snippet } from 'svelte';
	import RawMarker from './RawMarker.svelte';

	let { children }: { children: Snippet } = $props();
</script>

<li class="align-middle">
	<RawMarker text="- " />
	{@render children()}
</li>
