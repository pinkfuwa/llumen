<script lang="ts">
	import { Markdown } from '$lib/components/markdown';

	let { content, streaming = false }: { content: string; streaming?: boolean } = $props();
</script>

<div class="space-y-2">
	<Markdown source={content} incremental={streaming} copy />
</div>
