<script lang="ts">
	import { copy } from '$lib/copy';
	import Latex from '../latex/Latex.svelte';

	let { node }: { node: { text?: string } } = $props();
</script>

<div class="flex justify-center">
	<button
		onclick={() => copy(node.text!)}
		class="overflow-x-auto rounded-md border border-outline p-4"
	>
		<Latex text={node.text!} displayMode />
	</button>
</div>
