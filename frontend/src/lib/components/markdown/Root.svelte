<script lang="ts">
	import { untrack } from 'svelte';
	import Parser from './Parser.svelte';
	import { parseMarkdown } from './worker/caller';
	import { parseIncremental, patchASTNodes, type IncrementalState } from './incremental';
	import type { AstNode } from './parser/types';

	const { source, incremental = false }: { source: string; incremental?: boolean } = $props();

	let incrementalState: Partial<IncrementalState> = $state({});
	let children: { children: AstNode[] } = $state({ children: [] });
	let nodes = $derived(children.children);

	let throttleTimer: ReturnType<typeof setTimeout> | null = null;
	const THROTTLE_MS = 100;

	let pendingSource: string | null = null;

	function doIncrementalParse(currentSource: string) {
		const currentState = untrack(() => incrementalState);

		try {
			let result = parseIncremental(currentSource, currentState);
			patchASTNodes(children, { children: result });
		} catch (error) {
			console.error('Incremental parse error:', error);
			parseMarkdown(currentSource)
				.then((r) => {
					children.children = r;
				})
				.catch((e) => console.error('Fallback parse error:', e));
		}
	}

	async function doFullParse(currentSource: string) {
		try {
			children.children = await parseMarkdown(currentSource);
		} catch (error) {
			console.error('Parse error:', error);
			children.children = [];
		}
	}

	$effect(() => {
		pendingSource = source;

		if (incremental) {
			if (!throttleTimer) {
				const runThrottle = () => {
					const lastParsedSource = untrack(() => pendingSource);
					if (lastParsedSource !== null) {
						doIncrementalParse(lastParsedSource);
					}

					const currentPending = untrack(() => pendingSource);
					if (currentPending !== lastParsedSource) {
						throttleTimer = setTimeout(runThrottle, THROTTLE_MS);
					} else {
						throttleTimer = null;
					}
				};
				throttleTimer = setTimeout(runThrottle, THROTTLE_MS);
			}
		} else {
			incrementalState = {};

			if (throttleTimer != null) {
				clearTimeout(throttleTimer);
				throttleTimer = null;
			}
			doFullParse(source);
		}
	});
</script>

{#if nodes.length == 0}
	<div class="space-y-2 whitespace-pre-wrap">
		{source}
	</div>
{:else}
	<div class="space-y-2">
		<Parser {nodes} />
	</div>
{/if}
