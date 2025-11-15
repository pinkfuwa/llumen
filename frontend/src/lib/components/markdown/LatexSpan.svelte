<script lang="ts">
	import { copy } from '$lib/copy';

	import Latex from '../latex/Latex.svelte';

	let { node }: { node: ASTNode } = $props();
</script>

<button
	onclick={() => copy(node.text!)}
	class="max-w-full overflow-x-auto rounded-md p-2 font-semibold"
>
	<Latex text={node.text!} />
</button>
