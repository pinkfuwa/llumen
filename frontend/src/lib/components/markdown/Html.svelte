<script lang="ts">
	let { node } = $props();
</script>

{#if node.text.trim() == '<br>'}
	<br />
{:else}
	{node.text}
{/if}
