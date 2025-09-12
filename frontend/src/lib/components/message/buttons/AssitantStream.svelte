<script lang="ts">
	import Parser from '$lib/components/markdown/Parser.svelte';
	import type { TokensList } from 'marked';

	const { list }: { list: TokensList[] } = $props();
</script>

<div class="space-y-2">
	{#each list as tokens, i}
		<Parser {tokens} monochrome />
	{/each}
</div>
