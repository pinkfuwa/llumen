<script lang="ts">
	import { untrack } from 'svelte';
	import Parser from './Parser.svelte';
	import { parseIncremental, walkTree } from './lexer';
	import { parseAst } from './worker';

	const { source, monochrome = false, incremental = false } = $props();

	let prevSource: string = $state('');
	let prevTree: import('@lezer/common').Tree | null = null;
	let ast: any = $state(null);

	$effect(() => {
		const prevTreeValue = untrack(() => prevTree);
		const prevSourceValue = untrack(() => prevSource);

		async function updateAst() {
			const increment =
				incremental &&
				prevTreeValue &&
				source.startsWith(prevSourceValue) &&
				source.length > prevSourceValue.length;

			if (increment) {
				const tree = await parseIncremental(prevTreeValue, prevSourceValue, source);
				ast = await walkTree(tree, source);
				prevSource = source;
				prevTree = tree;
			} else {
				ast = await parseAst(source);
			}
		}

		updateAst();
	});
</script>

{#if ast == null}
	<div class="space-y-2">
		{#each source.split('\n') as line}
			<p>{line}</p>
		{/each}
	</div>
{:else}
	<div class="space-y-2">
		<Parser {ast} {monochrome} />
	</div>
{/if}
