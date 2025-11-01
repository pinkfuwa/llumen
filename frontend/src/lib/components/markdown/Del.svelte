<script lang="ts">
	let { node, monochrome = false, children } = $props();
</script>

<del class:monochrome>
	{@render children?.()}
</del>
