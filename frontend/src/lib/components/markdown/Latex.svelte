<script lang="ts">
	import { copy } from '$lib/copy';
	import Latex from '../latex/Latex.svelte';

	let { node } = $props<{ node: { text: string } }>();
</script>

<div class="flex justify-center overflow-x-auto">
	<button onclick={() => copy(node.text)} class="rounded-md border border-outline p-4">
		<Latex text={node.text} displayMode />
	</button>
</div>
