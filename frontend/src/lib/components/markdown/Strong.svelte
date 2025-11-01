<script lang="ts">
	let { node, monochrome = false, children } = $props();
</script>

<strong>
	{@render children?.()}
</strong>
