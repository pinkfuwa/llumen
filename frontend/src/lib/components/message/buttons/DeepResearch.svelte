<script lang="ts">
	import { Atom } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import { Collapsible } from 'bits-ui';
	import ResearchStep from './ResearchStep.svelte';
	import type { Deep, Step } from '$lib/api/types';
	let {
		plan,
		open = $bindable(false),
		streaming = false
	} = $props<{ plan: Deep; open?: boolean; streaming?: boolean }>();
</script>

<Collapsible.Root bind:open>
	<Collapsible.Trigger
		class="flex flex-row flex-nowrap rounded p-2 duration-150 hover:bg-primary hover:text-text-hover"
	>
		<Atom class="mr-2" />
		<span> Research Agent </span>
	</Collapsible.Trigger>
	<Collapsible.Content
		class="py-2 pl-3 slide-out-to-start-2 fade-in fade-out slide-in-from-top-2 data-[state=close]:animate-out data-[state=open]:animate-in"
	>
		<div class="text-xl font-bold">
			{plan.title}
		</div>
		{#each plan.steps as step}
			<ResearchStep step={step as Step} {streaming} />
		{/each}
	</Collapsible.Content>
</Collapsible.Root>
