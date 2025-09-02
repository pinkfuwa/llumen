<script lang="ts">
	import type { TokensList } from 'marked';
	import Parser from '$lib/markdown/Parser.svelte';
	import { _ } from 'svelte-i18n';
	import AssitantBox from './buttons/AssitantBox.svelte';

	const { list }: { list: TokensList[] } = $props();
</script>

<AssitantBox>
	{#each list as tokens, i}
		<Parser {tokens} monochrome />
	{/each}
</AssitantBox>
