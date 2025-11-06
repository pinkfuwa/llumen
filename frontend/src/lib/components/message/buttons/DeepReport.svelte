<script lang="ts">
	import { Markdown } from '$lib/components/markdown';
	import { JSONParser } from '@streamparser/json-whatwg';

	let { content, streaming = false }: { content: string; streaming?: boolean } = $props();
	
	let everStream = $state(false);
	$effect(() => {
		everStream = streaming || everStream;
	});

	let reportContent = $state('');

	// Parse incrementally using streaming parser
	$effect(() => {
		try {
			// Try to parse as complete JSON first
			const parsed = JSON.parse(content);
			reportContent = parsed.content || content;
		} catch {
			// If incomplete, try incremental parsing or use raw content
			try {
				const parser = new JSONParser();
				let partialReport = { content: '' };
				
				parser.onValue = ({ value, key, parent, stack }) => {
					if (stack.length === 0 && typeof value === 'object' && value !== null) {
						// Root level complete object
						partialReport = value as any;
					} else if (key === 'content') {
						partialReport.content = value as string;
					}
				};
				
				parser.write(content);
				reportContent = partialReport.content || content;
			} catch {
				// If all parsing fails, use raw content
				reportContent = content;
			}
		}
	});
</script>

<div class="my-4 rounded-lg border border-gray-200 dark:border-gray-700 p-4 bg-white dark:bg-gray-900">
	<Markdown source={reportContent} incremental={everStream} />
</div>
