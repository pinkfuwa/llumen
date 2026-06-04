<script lang="ts">
	import type { TextNode } from '../vendor/types';

	let { node }: { node: TextNode } = $props();

	const content = $derived(node.content || '');
</script>

<span class="align-middle">
	{content}
</span>
