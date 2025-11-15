<script lang="ts">
	let { node }: { node: ASTNode } = $props();
</script>

<span>{node.text}</span>
