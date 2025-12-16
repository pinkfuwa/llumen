<script lang="ts">
	import type { TextToken } from './lexer';

	let { token }: { token: TextToken } = $props();

	const content = $derived(token.content || '');
</script>

{content}
