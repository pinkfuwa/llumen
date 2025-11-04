<script lang="ts">
	import { AlertCircle, CheckCircle, Loader2 } from '@lucide/svelte';

	let { content }: { content: string } = $props();

	let step = $derived.by(() => {
		try {
			return JSON.parse(content);
		} catch {
			return { id: '', description: '', status: 'pending', result: null };
		}
	});

	function getStatusColor(status: string) {
		switch (status) {
			case 'completed':
				return 'bg-green-100 dark:bg-green-900/30 border-green-300 dark:border-green-700';
			case 'in_progress':
				return 'bg-blue-100 dark:bg-blue-900/30 border-blue-300 dark:border-blue-700';
			case 'failed':
				return 'bg-red-100 dark:bg-red-900/30 border-red-300 dark:border-red-700';
			default:
				return 'bg-gray-100 dark:bg-gray-800 border-gray-300 dark:border-gray-700';
		}
	}

	function getStatusIcon(status: string) {
		switch (status) {
			case 'completed':
				return CheckCircle;
			case 'in_progress':
				return Loader2;
			case 'failed':
				return AlertCircle;
			default:
				return Loader2;
		}
	}
</script>

<div class="my-3 rounded-lg border {getStatusColor(step.status)} p-3">
	<div class="flex items-center gap-2 mb-2">
		{@const Icon = getStatusIcon(step.status)}
		<Icon
			class="w-4 h-4 {step.status === 'in_progress' ? 'animate-spin' : ''} {step.status ===
			'completed'
				? 'text-green-600 dark:text-green-400'
				: step.status === 'failed'
					? 'text-red-600 dark:text-red-400'
					: 'text-blue-600 dark:text-blue-400'}"
		/>
		<span class="text-sm font-medium text-gray-900 dark:text-gray-100">
			{step.description}
		</span>
	</div>
	{#if step.result}
		<div class="text-sm text-gray-700 dark:text-gray-300 ml-6">
			{step.result}
		</div>
	{/if}
</div>
