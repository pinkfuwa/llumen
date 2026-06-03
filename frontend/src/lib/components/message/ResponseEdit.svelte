<script lang="ts">
	import { copy } from '$lib/copy.svelte';
	import { CircleDollarSign, ClipboardCopy } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	

	let { content = '', token = 0, cost = 0.0 } = $props();
	let showUsage = $derived(token > 0);
</script>

<div class="flex justify-end space-x-1 duration-150 group-hover:visible md:invisible">
	<div class="group/usage relative flex space-x-1">
		{#if showUsage}
			<CircleDollarSign
				class="h-10 w-10 rounded-lg p-2 duration-150  group-hover/usage:bg-interactive-hover"
			/>

			<div
				class="absolute top-0 right-13 flex h-10 w-sm items-center justify-end duration-150 group-hover/usage:visible md:invisible"
			>
				<div class="rounded-md bg-muted p-2 select-none">
					{token} token/${cost.toFixed(4)}
				</div>
			</div>
		{/if}
	</div>
	<div
		role="button"
		tabindex="0"
		onclick={() => copy(content)}
		onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); copy(content); } }}
		aria-label="copy response"
		class="cursor-pointer duration-150 hover:bg-interactive-hover h-10 w-10 rounded-lg"
	>
		<ClipboardCopy class="h-10 w-10 rounded-lg p-2 duration-150 hover:bg-interactive-hover" />
	</div>
</div>
