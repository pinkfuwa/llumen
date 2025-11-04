<script lang="ts">
	import { CheckCircle, Circle, Loader2 } from '@lucide/svelte';

	let { content }: { content: string } = $props();

	let plan = $derived.by(() => {
		try {
			return JSON.parse(content);
		} catch {
			return { steps: [], has_enough_context: false };
		}
	});

	function getStatusIcon(status: string) {
		switch (status) {
			case 'completed':
				return CheckCircle;
			case 'in_progress':
				return Loader2;
			case 'failed':
				return Circle;
			default:
				return Circle;
		}
	}

	function getStatusClass(status: string) {
		switch (status) {
			case 'completed':
				return 'text-green-600 dark:text-green-400';
			case 'in_progress':
				return 'text-blue-600 dark:text-blue-400 animate-spin';
			case 'failed':
				return 'text-red-600 dark:text-red-400';
			default:
				return 'text-gray-600 dark:text-gray-400';
		}
	}
</script>

<div class="my-4 rounded-lg border border-gray-200 dark:border-gray-700 p-4 bg-gray-50 dark:bg-gray-800/50">
	<h3 class="text-lg font-semibold mb-3 text-gray-900 dark:text-gray-100">Research Plan</h3>
	<div class="space-y-2">
		{#each plan.steps as step}
			<div class="flex items-start gap-2">
				{@const Icon = getStatusIcon(step.status)}
				<Icon class="w-5 h-5 mt-0.5 flex-shrink-0 {getStatusClass(step.status)}" />
				<div class="flex-1">
					<p class="text-sm text-gray-700 dark:text-gray-300">{step.description}</p>
					{#if step.need_search}
						<span class="text-xs text-gray-500 dark:text-gray-400">Requires web search</span>
					{/if}
				</div>
			</div>
		{/each}
	</div>
	{#if !plan.has_enough_context}
		<p class="mt-3 text-sm text-gray-600 dark:text-gray-400 italic">
			Gathering additional information...
		</p>
	{/if}
</div>
