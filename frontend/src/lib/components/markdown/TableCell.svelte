<script lang="ts">
	import Root from './Root.svelte';

	let { node, children } = $props();
</script>

<td class="px-6 py-4 font-semibold">
	<div>
		{@render children?.()}
	</div>
</td>
