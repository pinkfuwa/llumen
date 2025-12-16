<script lang="ts">
	import { Markdown } from '$lib/components/markdown';

	let { content, streaming = false }: { content: string; streaming?: boolean } = $props();
	let everStream = $state(false);
	$effect(() => {
		everStream = streaming || everStream;
	});
</script>

<div class="space-y-2">
	<Markdown source={content} incremental={everStream} monochrome={streaming} />
</div>
