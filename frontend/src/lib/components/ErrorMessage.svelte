<script lang="ts">
	import { CircleX } from '@lucide/svelte';
	import { fade } from 'svelte/transition';

	let { kind, reason = undefined }: { kind: string; reason: undefined | string } = $props();

	let open = $state(true);
</script>

{#if open}
	<button
		class="fixed top-0 right-0 z-6 m-3 rounded-md border border-outline bg-background px-3 py-2 text-left"
		in:fade={{ duration: 150 }}
		out:fade={{ duration: 150 }}
		onmouseleave={() => (open = false)}
		onclick={() => (open = false)}
	>
		<div class="mb-2 flex items-center">
			<CircleX class="mr-2 inline-block" />
			{kind} error
		</div>
		{#if reason}
			<div class="max-w-sm lg:max-w-lg">
				{reason}
			</div>
		{/if}
	</button>
{/if}
