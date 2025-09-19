<script lang="ts">
	import { ToolCase } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import { slide } from 'svelte/transition';

	let { children, toolname = 'Default Tool' } = $props();

	let open = $state(false);
</script>

<button onclick={() => (open = !open)} class="w-full text-left">
	<div class="w-full border-l-6 border-outline py-1 pr-6 pl-4">
		<div class="mb-2 flex items-center">
			<ToolCase class="mr-2" />
			{#if !open}
				<span class="mr-1"> Calling </span>
			{/if}
			<span class="rounded-md bg-primary px-2 py-[2px]">
				{toolname}
			</span>
		</div>
		{#if open}
			<div in:slide={{ duration: 180, axis: 'y' }} out:slide={{ duration: 180, axis: 'y' }}>
				{@render children()}
			</div>
		{/if}
	</div>
</button>
