<script lang="ts">
	import Root from '$lib/components/markdown/Root.svelte';

	let { content, monochrome = false }: { content: string; monochrome: boolean } = $props();
</script>

<div class="space-y-2">
	<Root source={content} {monochrome} />
</div>
