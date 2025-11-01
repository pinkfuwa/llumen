<script lang="ts">
	let { node, children } = $props();

	const classMap: Record<string, string> = {
		ATXHeading1: 'mt-1 text-2xl font-bold',
		SetextHeading1: 'mt-1 text-2xl font-bold',
		ATXHeading2: 'mt-1 text-xl font-bold',
		SetextHeading2: 'mt-1 text-xl font-bold',
		ATXHeading3: 'mt-1 text-lg font-bold'
	};
</script>

<h2 class={classMap[node.type]}>
	{@render children?.()}
</h2>
