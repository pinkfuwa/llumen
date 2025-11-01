<script lang="ts">
	let { monochrome = false, children } = $props();
</script>

<tr class:monochrome>
	{@render children?.()}
</tr>
