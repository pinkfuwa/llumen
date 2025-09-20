<script lang="ts">
	import { copy } from '$lib/copy';
	import { CircleDollarSign, ClipboardCopy } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';

	let { content = '', token = 0, cost = 0.0 } = $props();
</script>

<div class="flex justify-end space-x-1 opacity-0 duration-150 group-hover:opacity-100">
	<div class="group/usage relative flex space-x-1">
		<CircleDollarSign
			class="h-10 w-10 rounded-lg p-2 duration-150  group-hover/usage:bg-primary group-hover/usage:text-text-hover"
		/>

		<div
			class="absolute top-0 right-13 flex h-10 w-sm items-center justify-end opacity-0 duration-150 group-hover/usage:opacity-100"
		>
			<div class="rounded-md bg-secondary p-2 select-none">
				{token} token/${cost.toFixed(4)}
			</div>
		</div>
	</div>
	<button onclick={() => copy(content)} aria-label="copy response">
		<ClipboardCopy class="h-10 w-10 rounded-lg p-2 hover:bg-primary hover:text-text-hover duration-150" />
	</button>
</div>
