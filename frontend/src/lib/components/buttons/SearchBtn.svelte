<script lang="ts">
	type Stage = 0 | 1 | 2;
	let { value = $bindable(0) as Stage } = $props();
	import { Tooltip } from '@svelte-plugins/tooltips';
	import { Atom, SearchCode, ZapOff } from '@lucide/svelte';

	function nextStage() {
		value = (value + 1) % 3;
	}
</script>

<button onclick={nextStage} class="rounded-md bg-background p-1 hover:bg-hover">
	{#if value == 2}
		<Tooltip content="In depth research on complex topic">
			<Atom class="inline-block" />
		</Tooltip>
	{:else if value == 1}
		<Tooltip content="Web enabled Chat">
			<SearchCode class="inline-block" />
		</Tooltip>
	{:else}
		<Tooltip content="Normal Chat">
			<ZapOff class="inline-block" />
		</Tooltip>
	{/if}
</button>
