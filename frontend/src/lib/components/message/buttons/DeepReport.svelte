<script lang="ts">
	import { Markdown } from '$lib/components/markdown';

	let { content, streaming = false }: { content: string; streaming?: boolean } = $props();
	
	let everStream = $state(false);
	$effect(() => {
		everStream = streaming || everStream;
	});

	let reportContent = $derived.by(() => {
		try {
			const parsed = JSON.parse(content);
			return parsed.content || content;
		} catch {
			return content;
		}
	});
</script>

<div class="my-4 rounded-lg border border-gray-200 dark:border-gray-700 p-4 bg-white dark:bg-gray-900">
	<Markdown source={reportContent} incremental={everStream} />
</div>
