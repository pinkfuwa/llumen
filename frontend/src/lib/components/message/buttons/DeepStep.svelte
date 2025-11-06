<script lang="ts">
	import { AlertCircle, CheckCircle, Loader2 } from '@lucide/svelte';
	import { JSONParser } from '@streamparser/json-whatwg';

	let { content }: { content: string } = $props();

	let step = $state<any>({ id: '', description: '', status: 'in_progress', result: null });

	// Parse incrementally using streaming parser
	$effect(() => {
		try {
			// Try to parse as complete JSON first
			const parsed = JSON.parse(content);
			step = parsed;
		} catch {
			// If incomplete, try incremental parsing
			try {
				const parser = new JSONParser();
				let partialStep = { id: '', description: '', status: 'in_progress', result: null };
				
				parser.onValue = ({ value, key, parent, stack }) => {
					if (stack.length === 0) {
						// Root level complete object
						partialStep = value as any;
					} else if (key === 'id') {
						partialStep.id = value as string;
					} else if (key === 'description') {
						partialStep.description = value as string;
					} else if (key === 'status') {
						partialStep.status = value as string;
					} else if (key === 'result') {
						partialStep.result = value as string | null;
					}
				};
				
				parser.write(content);
				step = partialStep;
			} catch {
				// Keep existing step if parsing fails
			}
		}
	});

	// change type to keyof
	const iconMap: Record<string, typeof CheckCircle> = {
		completed: CheckCircle,
		in_progress: Loader2,
		failed: AlertCircle
	};

	const Icon = $derived(iconMap[step.status] || Loader2);
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
