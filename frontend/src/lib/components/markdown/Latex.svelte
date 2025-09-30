<script lang="ts">
	import { copy } from '$lib/copy';
	import Latex from '../latex/Latex.svelte';

	let { raw } = $props<{ raw: string }>();
</script>

<div class="flex justify-center overflow-x-auto">
	<button onclick={() => copy(raw)} class="rounded-md border border-outline p-4">
		<Latex text={raw} displayMode />
	</button>
</div>
