<script lang="ts">
	import { CheckCircle, Circle, Loader2 } from '@lucide/svelte';
	import { JSONParser } from '@streamparser/json-whatwg';

	let { content }: { content: string } = $props();

	let plan = $state<any>({ steps: [], has_enough_context: false });

	// Parse incrementally using streaming parser
	$effect(() => {
		try {
			// Try to parse as complete JSON first
			const parsed = JSON.parse(content);
			plan = parsed;
		} catch {
			// If incomplete, try incremental parsing
			try {
				const parser = new JSONParser();
				let partialPlan = { steps: [], has_enough_context: false };
				
				parser.onValue = ({ value, key, stack }) => {
					if (stack.length === 0) {
						// Root level complete object
						partialPlan = value as any;
					} else if (key === 'steps' && Array.isArray(value)) {
						partialPlan.steps = value;
					} else if (key === 'has_enough_context') {
						partialPlan.has_enough_context = value as boolean;
					}
				};
				
				parser.write(content);
				plan = partialPlan;
			} catch {
				// Keep existing plan if parsing fails
			}
		}
	});

	// change type to keyof
	const iconMap: Record<string, typeof Circle> = {
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
			{@const Icon = iconMap[step.status] || Circle}
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
