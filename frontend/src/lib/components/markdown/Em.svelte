<script lang="ts">
	let { node, monochrome = false, children } = $props();
</script>

<em class:monochrome>
	{@render children?.()}
</em>
