<script lang="ts">
	import Latex from '../latex/Latex.svelte';

	let { raw } = $props<{ raw: string }>();
</script>

<div class="flex justify-center overflow-x-auto">
	<div class="rounded-md border border-outline p-4">
		<Latex text={raw} displayMode />
	</div>
</div>
