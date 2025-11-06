<script lang="ts">
	import { AlertCircle, CheckCircle, Loader2 } from '@lucide/svelte';

	interface Step {
		id: string;
		description: string;
		status: 'completed' | 'in_progress' | 'failed';
		result: string | null;
	}

	let { content }: { content: string } = $props();

	let step = $state<Step>({
		id: '',
		description: '',
		status: 'in_progress',
		result: null
	});

	// Parse incrementally
	$effect(() => {
		try {
			// Try to parse as complete JSON first
			const parsed = JSON.parse(content);
			step = parsed as Step;
		} catch {
			// If incomplete, try to extract fields from partial JSON
			try {
				const idMatch = content.match(/"id"\s*:\s*"([^"]*)"/);
				const descMatch = content.match(/"description"\s*:\s*"([^"]*)"/);
				const statusMatch = content.match(/"status"\s*:\s*"(completed|in_progress|failed)"/);
				const resultMatch = content.match(/"result"\s*:\s*"([^"]*)"/);

				step = {
					id: idMatch?.[1] ?? '',
					description: descMatch?.[1] ?? '',
					status: (statusMatch?.[1] as 'completed' | 'in_progress' | 'failed') ?? 'in_progress',
					result: resultMatch?.[1] ?? null
				};
			} catch {
				// Keep existing step if parsing fails
			}
		}
	});

	const iconMap: Record<'completed' | 'in_progress' | 'failed', typeof CheckCircle> = {
		completed: CheckCircle,
		in_progress: Loader2,
		failed: AlertCircle
	};

	const Icon = $derived(iconMap[step.status]);
</script>

<div
	class="my-3 rounded-lg border p-3
		data-[status=completed]:border-green-300 data-[status=completed]:bg-green-100 data-[status=failed]:border-red-300 data-[status=failed]:bg-red-100
		data-[status=in_progress]:border-blue-300 data-[status=in_progress]:bg-blue-100 data-[status=completed]:dark:border-green-700 data-[status=completed]:dark:bg-green-900/30
		data-[status=failed]:dark:border-red-700 data-[status=failed]:dark:bg-red-900/30 data-[status=in_progress]:dark:border-blue-700 data-[status=in_progress]:dark:bg-blue-900/30"
	data-status={step.status}
>
	<div class="mb-2 flex items-center gap-2">
		<Icon
			class="h-4 w-4
				data-[status=completed]:text-[var(--color-primary)]
				data-[status=failed]:text-red-600
				data-[status=in_progress]:animate-spin data-[status=in_progress]:text-blue-600
				data-[status=failed]:dark:text-red-400 data-[status=in_progress]:dark:text-blue-400"
			data-status={step.status}
		/>
		<span class="text-sm font-medium text-gray-900 dark:text-gray-100">
			{step.description}
		</span>
	</div>
	<div
		class="ml-6 text-sm text-gray-700 data-[show=false]:hidden dark:text-gray-300"
		data-show={!!step.result}
	>
		{step.result || ''}
	</div>
</div>
