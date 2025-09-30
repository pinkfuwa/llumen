<script lang="ts">
	import Latex from '../latex/Latex.svelte';

	let { raw } = $props<{ raw: string }>();
</script>

<span class="rounded-md p-2 font-semibold">
	<Latex text={raw} />
</span>
