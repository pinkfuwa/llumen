<script lang="ts">
	let { value } = $props();
	import { Tooltip } from '@svelte-plugins/tooltips';
	import { Atom, SearchCode, ZapOff } from '@lucide/svelte';

	// stage 0: normal
	// stage 1: search
	// stage 2: deep
	const maxStage = 2;

	function nextStage() {
		value = value == maxStage ? 0 : value + 1;
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
