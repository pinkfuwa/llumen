<script lang="ts">
	import type { TextNode } from '../parser/types';

	let { node }: { node: TextNode } = $props();

	const content = $derived(node.content || '');
</script>

<span>
	{content}
</span>
