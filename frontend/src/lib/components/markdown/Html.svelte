<script lang="ts">
	import Code from '../shiki/Code.svelte';
	let { node, monochrome = false } = $props();
</script>

{#if node.text.trim() == '<br>'}
	<br />
{:else}
	<Code lang="html" text={node.text} {monochrome} />
{/if}
