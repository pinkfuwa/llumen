<script lang="ts">
	import Parser from './Parser.svelte';
	import { parseMarkdown, type WorkerToken } from './parser/index';
	import { parseMarkdown as workerParseMarkdown } from './worker';

	let { source, monochrome = false, incremental = false } = $props();

	let tokens: WorkerToken[] | undefined = $state(undefined);

	$effect(() => {
		(incremental ? parseMarkdown : workerParseMarkdown)(source).then((resp) => {
			tokens = resp;
		});
	});
	$inspect(tokens);
</script>

{#if tokens === undefined}
	{#each source.split('\n') as line}
		<p>{line}</p>
	{/each}
{:else}
	<Parser {tokens} {monochrome} />
{/if}
