<script lang="ts">
	import Code from '$lib/components/shiki/Code.svelte';

	let { content }: { content: string } = $props();

	let trimmedContent = $derived.by(() => {
		let trim = content.trim();
		let lines = trim.split('\n');
		if (lines.length > 10) return lines.slice(0, 10).join('\n');
		return trim;
	});
</script>

<div class="mt-1 space-y-2">
	<Code lang="text" text={trimmedContent.trim()} />
</div>
