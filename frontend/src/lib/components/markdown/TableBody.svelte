<script lang="ts">
	let { node, monochrome = false, children } = $props();
</script>

body
<tbody class:monochrome>
	{@render children?.()}
</tbody>
