<script lang="ts">
	import type { Token } from './lexer';
	import type { Snippet } from 'svelte';

	let { token, source, children }: { token: Token; source: string; children: Snippet } = $props();
</script>

<del>
	{@render children()}
</del>
