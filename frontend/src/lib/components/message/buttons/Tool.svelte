<script lang="ts">
	import Code from '../../shiki/Code.svelte';
	let { content }: { content: string } = $props();
</script>

{#if content.trim().length != 0}
	<Code lang="json" text={content} />
{/if}
