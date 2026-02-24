<script lang="ts">
	import type { LatexBlockToken } from './lexer';
	import LatexComponent from '../latex/Latex.svelte';

	let { token }: { token: LatexBlockToken } = $props();

	const content = $derived(token.content);
</script>

<div class="flex justify-center">
	<div class="max-w-full overflow-x-auto p-1.5 text-lg">
		<LatexComponent text={content} displayMode />
	</div>
</div>
