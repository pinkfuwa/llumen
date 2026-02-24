<script lang="ts">
	import type { LatexInlineToken } from './lexer';
	import LatexComponent from '../latex/Latex.svelte';

	let { token }: { token: LatexInlineToken } = $props();

	const content = $derived(token.content);
</script>

<span class="inline-block p-1 align-middle">
	<span class="inline-block max-w-full overflow-x-auto align-middle text-lg">
		<LatexComponent text={content} />
	</span>
</span>
