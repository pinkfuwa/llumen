<script lang="ts">
	import type { Deep } from '$lib/api/types';

	let { plan } = $props<{ plan: Deep }>();
	let lines = $derived(JSON.stringify(plan, null, 2).split('\n'));
</script>

<div>
	{#each lines as line}
		<p>{line}</p>
	{/each}
</div>
