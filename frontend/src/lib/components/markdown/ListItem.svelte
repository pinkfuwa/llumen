<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<li class="items-center">
	{@render children?.()}
</li>
