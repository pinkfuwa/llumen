<script lang="ts">
	import { Trash, TriangleAlert } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';

	let { ondelete }: { ondelete: () => void } = $props();
	let checked = $state(false);
</script>

<button
	onclick={() => {
		if (!checked) checked = true;
		else ondelete();
	}}
	onmouseleave={() => (checked = false)}
>
	{#if checked}
		<TriangleAlert class="h-10 w-10 rounded-lg p-2 hover:bg-hover" />
	{:else}
		<Trash class="h-10 w-10 rounded-lg p-2 hover:bg-hover" />
	{/if}
</button>
