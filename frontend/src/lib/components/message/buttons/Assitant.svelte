<script lang="ts">
	import { Markdown } from '$lib/components/markdown';

	let { content, monochrome = false }: { content: string; monochrome?: boolean } = $props();
</script>

<div class="space-y-2">
	<Markdown source={content} {monochrome} incremental={monochrome} />
</div>
