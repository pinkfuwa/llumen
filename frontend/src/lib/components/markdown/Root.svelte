<script lang="ts">
	import { untrack } from 'svelte';
	import Parser from './Parser.svelte';
	import { parseIncremental, walkTree } from './lexer';
	import { parseAst } from './worker';

	const { source, monochrome = false, incremental = false } = $props();

	let prevSource: string = $state('');
	let prevTree: import('@lezer/common').Tree | null = null;
	let ast: any = $state(null);

	let throttleTimer: ReturnType<typeof setTimeout> | null = null;
	const THROTTLE_MS = 100;

	let pendingSource: string | null = null;

	async function doParse(currentSource: string) {
		const prevTreeValue = untrack(() => prevTree);
		const prevSourceValue = untrack(() => prevSource);

		const increment =
			incremental &&
			prevTreeValue &&
			currentSource.startsWith(prevSourceValue) &&
			currentSource.length > prevSourceValue.length;

		if (increment) {
			const tree = await parseIncremental(prevTreeValue, prevSourceValue, currentSource);
			ast = await walkTree(tree, currentSource);
			prevSource = currentSource;
			prevTree = tree;
		} else {
			ast = await parseAst(currentSource);
			prevSource = currentSource;
			prevTree = null;
		}
	}

	$effect(() => {
		pendingSource = source;

		if (!throttleTimer) {
			const runThrottle = async () => {
				let lastParsedSource = pendingSource;
				await doParse(lastParsedSource!);

				if (pendingSource !== lastParsedSource) {
					throttleTimer = setTimeout(runThrottle, THROTTLE_MS);
				} else {
					throttleTimer = null;
				}
			};
			throttleTimer = setTimeout(runThrottle, THROTTLE_MS);
		}
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
