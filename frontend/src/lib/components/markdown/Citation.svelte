<script lang="ts">
	let { node, monochrome = false, children } = $props();
</script>

<cite class:monochrome>
	{@render children?.()}
</cite>
