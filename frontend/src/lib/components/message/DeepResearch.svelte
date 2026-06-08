<script lang="ts">
	import { Atom } from '@lucide/svelte';
	import { Collapsible } from 'bits-ui';
	import ResearchStep from './ResearchStep.svelte';
	import type { Deep, Step } from '$lib/api/types';
	import { t } from 'svelte-intl-precompile';

	let {
		plan,
		open = $bindable(false),
		streaming = false
	} = $props<{ plan: Deep; open?: boolean; streaming?: boolean }>();

	const triggerStyle =
		'flex flex-row flex-nowrap rounded p-2 cursor-pointer duration-150 hover:bg-interactive-hover';
</script>

<Collapsible.Root bind:open>
	<Collapsible.Trigger class={triggerStyle}>
		<Atom class="mr-2" />
		<span> {$t('chat.research_agent')} </span>
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
