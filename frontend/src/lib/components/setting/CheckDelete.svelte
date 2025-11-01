<script lang="ts">
	import { Trash, TriangleAlert } from '@lucide/svelte';
	import { Button } from 'bits-ui';
	import { _ } from 'svelte-i18n';

	let { ondelete }: { ondelete: () => void } = $props();
	let checked = $state(false);
</script>

<Button.Root
	onclick={(e: MouseEvent) => {
		e.stopPropagation();
		if (!checked) checked = true;
		else ondelete();
	}}
	onmouseleave={() => (checked = false)}
>
	{#if checked}
		<TriangleAlert class="h-10 w-10 rounded-lg p-2" />
	{:else}
		<Trash class="h-10 w-10 rounded-lg p-2" />
	{/if}
</Button.Root>
