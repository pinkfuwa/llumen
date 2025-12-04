<script lang="ts">
	import { StepKind, type Step } from '$lib/api/types';
	import { FlaskConical, ChartSpline, TextSearch } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import { Collapsible } from 'bits-ui';
	import { Markdown } from '$lib/components/markdown';
	let {
		step,
		open = $bindable(false),
		streaming = false
	}: { step: Step; open: boolean; streaming: boolean } = $props<{
		step: Step;
		open?: boolean;
		streaming?: boolean;
	}>();

	let lastProgress = $derived(step.progress.filter((m) => m.t == 'text').at(-1));
</script>

<Collapsible.Root bind:open>
	<Collapsible.Trigger
		class="flex flex-row flex-nowrap rounded p-1 duration-150 hover:bg-primary hover:text-text-hover"
	>
		{#if step.kind == StepKind.Code}
			<ChartSpline class="mr-2" />
		{:else if step.need_search}
			<TextSearch class="mr-2" />
		{:else}
			<FlaskConical class="mr-2" />
		{/if}
		<span> {step.title} </span>
	</Collapsible.Trigger>
	<Collapsible.Content
		class="space-y-2 py-1 pl-3 slide-out-to-start-2 fade-in fade-out slide-in-from-top-2 data-[state=close]:animate-out data-[state=open]:animate-in"
	>
		{step.description}

		<div class="my-1 space-y-1 border-l-4 pl-3 hover:border-primary">
			{#if lastProgress != undefined}
				<Markdown source={lastProgress.c || ''} incremental={streaming} />
			{/if}
		</div>
	</Collapsible.Content>
</Collapsible.Root>
