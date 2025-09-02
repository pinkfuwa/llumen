<script lang="ts">
	import type { TokensList } from 'marked';
	import Parser from '$lib/markdown/Parser.svelte';
	import { _ } from 'svelte-i18n';
	import ResponseBox from './buttons/ResponseBox.svelte';

	const { list }: { list: TokensList[] } = $props();
</script>

<ResponseBox>
	{#each list as tokens, i}
		<Parser {tokens} monochrome />
	{/each}
</ResponseBox>
