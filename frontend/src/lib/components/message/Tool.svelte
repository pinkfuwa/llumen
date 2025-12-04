<script lang="ts">
	import Code from '$lib/components/shiki/Code.svelte';
	let { content }: { content: string } = $props();
</script>

{#if content.trim().length != 0}
	<Code lang="json" text={content} />
{/if}
