<script lang="ts">
	import { CheckCircle, Circle, Loader2 } from '@lucide/svelte';

	interface PlanStep {
		description: string;
		status: 'completed' | 'in_progress' | 'failed';
		need_search?: boolean;
	}

	interface Plan {
		steps: PlanStep[];
		has_enough_context: boolean;
	}

	let { content }: { content: string } = $props();

	const defaultPlan: Plan = {
		steps: [],
		has_enough_context: false
	};

	let plan = $state<Plan>(defaultPlan);

	// Parse content and extract plan information
	$effect(() => {
		try {
			const parsed = JSON.parse(content);
			if (parsed && typeof parsed === 'object') {
				const steps = Array.isArray(parsed.steps)
					? parsed.steps.map((step: unknown) => {
							if (step && typeof step === 'object') {
								const stepObj = step as Record<string, unknown>;
								return {
									description: String(stepObj.description || ''),
									status: (['completed', 'in_progress', 'failed'].includes(String(stepObj.status))
										? String(stepObj.status)
										: 'in_progress') as PlanStep['status'],
									need_search: Boolean(stepObj.need_search)
								};
							}
							return { description: '', status: 'in_progress' as const, need_search: false };
						})
					: [];

				plan = {
					steps,
					has_enough_context: Boolean(parsed.has_enough_context)
				};
			}
		} catch {
			// Keep existing plan if parsing fails
		}
	});

	const iconMap: Record<PlanStep['status'], typeof Circle> = {
		completed: CheckCircle,
		in_progress: Loader2,
		failed: Circle
	};
</script>

<div
	class="my-4 rounded-lg border border-gray-200 bg-gray-50 p-4 dark:border-gray-700 dark:bg-gray-800/50"
>
	<h3 class="mb-3 text-lg font-semibold text-gray-900 dark:text-gray-100">Research Plan</h3>
	<div class="space-y-2">
		{#each plan.steps as step}
			{@const Icon = iconMap[step.status]}
			<div class="flex items-start gap-2">
				<!-- FIXME: fix copilot's style with data-state= -->
				<Icon class="mt-0.5 h-5 w-5 flex-shrink-0" />
				<div class="flex-1">
					<p class="text-sm text-gray-700 dark:text-gray-300">{step.description}</p>
					<span
						class="text-xs text-gray-500 data-[show=false]:hidden dark:text-gray-400"
						data-show={step.need_search}>Requires web search</span
					>
				</div>
			</div>
		{/each}
	</div>
	<p
		class="mt-3 text-sm text-gray-600 italic data-[show=false]:hidden dark:text-gray-400"
		data-show={!plan.has_enough_context}
	>
		Gathering additional information...
	</p>
</div>
