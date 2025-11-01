<script lang="ts">
	let { node, children } = $props();
</script>

<li class="items-center">
	{@render children?.()}
</li>
