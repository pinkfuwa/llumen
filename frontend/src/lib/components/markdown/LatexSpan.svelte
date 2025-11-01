<script lang="ts">
	import { copy } from '$lib/copy';

	import Latex from '../latex/Latex.svelte';

	let { node, monochrome = false } = $props();
</script>

<button onclick={() => copy(node.text)} class="rounded-md p-2 font-semibold break-all">
	<Latex text={node.text} />
</button>
