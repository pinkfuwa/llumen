<script lang="ts">
	let { node }: { node: ASTNode } = $props();
</script>

{#if node.text == undefined}{:else if node.text.trim() == '<br>'}
	<br />
{:else}
	{node.text}
{/if}
