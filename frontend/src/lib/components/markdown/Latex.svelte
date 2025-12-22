<script lang="ts">
	import type { LatexBlockToken } from './lexer';
	import LatexComponent from '../latex/Latex.svelte';

	let { token }: { token: LatexBlockToken } = $props();

	const content = $derived(token.content);
</script>

<div class="my-4 flex justify-center">
	<div class="overflow-x-auto">
		<LatexComponent text={content} displayMode />
	</div>
</div>
