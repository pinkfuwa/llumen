<script lang="ts">
	let { node, monochrome = false, children } = $props();
</script>

<thead>{@render children?.()}</thead>
