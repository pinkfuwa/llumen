<script lang="ts">
	import { Markdown } from '$lib/components/markdown';

	let { content, streaming = false }: { content: string; streaming?: boolean } = $props();

	let everStream = $state(false);
	$effect(() => {
		everStream = streaming || everStream;
	});

	let reportContent = $state('');

	// Parse JSON content to extract the report
	$effect(() => {
		try {
			// Try to parse as complete JSON first
			const parsed = JSON.parse(content);
			reportContent =
				parsed && typeof parsed === 'object' && 'content' in parsed
					? String(parsed.content)
					: content;
		} catch {
			// If parsing fails, use raw content
			reportContent = content;
		}
	});
</script>

<div
	class="my-4 rounded-lg border border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-900"
>
	<Markdown source={reportContent} incremental={everStream} />
</div>
