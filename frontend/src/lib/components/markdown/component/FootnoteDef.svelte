<script lang="ts">
	import type { FootnoteDefNode } from '../parser/types';

	let { node }: { node: FootnoteDefNode } = $props();

	const label = $derived(node.label);
</script>

<span class="align-super text-[0.75em] font-medium text-markdown-link">[^{label}]</span>
