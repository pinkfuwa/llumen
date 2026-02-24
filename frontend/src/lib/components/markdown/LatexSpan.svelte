<script lang="ts">
	import type { LatexInlineToken } from './lexer';
	import LatexComponent from '../latex/Latex.svelte';

	let { token }: { token: LatexInlineToken } = $props();

	const content = $derived(token.content);
</script>

<span class="inline-block align-middle">
	<span class="inline-block align-middle max-w-full overflow-x-auto">
		<LatexComponent text={content} />
	</span>
</span>
