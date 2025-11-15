<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<td class="px-6 py-4 font-semibold">
	<div>
		{@render children?.()}
	</div>
</td>
