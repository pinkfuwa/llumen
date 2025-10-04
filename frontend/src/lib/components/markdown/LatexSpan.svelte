<script lang="ts">
	import { copy } from '$lib/copy';

	import Latex from '../latex/Latex.svelte';

	let { raw } = $props<{ raw: string }>();
</script>

<button onclick={() => copy(raw)} class="rounded-md p-2 font-semibold break-all">
	<Latex text={raw} />
</button>
