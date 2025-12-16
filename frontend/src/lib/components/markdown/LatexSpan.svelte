<script lang="ts">
	import type { LatexInlineToken } from './lexer';
	import LatexComponent from '../latex/Latex.svelte';

	let { token }: { token: LatexInlineToken } = $props();

	const content = $derived(token.content);
</script>

<span class="inline-block">
	<LatexComponent text={content} />
</span>
