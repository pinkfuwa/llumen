<script lang="ts">
	import { Trash } from '@lucide/svelte';
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
	<Trash class="h-10 w-10 rounded-lg p-2 hover:bg-hover" />
</button>
