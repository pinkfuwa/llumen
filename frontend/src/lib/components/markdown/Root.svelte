<script lang="ts">
	import { untrack } from 'svelte';
	import Parser from './Parser.svelte';
	import { parseMarkdown } from './worker/caller';
	import {
		parseIncremental,
		type IncrementalState
	} from './incremental';
	import type { ParseResult } from './parser/types';

	const { source, incremental = false }: { source: string; incremental?: boolean } = $props();

	let incrementalState: IncrementalState | null = $state(null);
	let result: ParseResult | null = $state(null);

	let throttleTimer: ReturnType<typeof setTimeout> | null = null;
	const THROTTLE_MS = 100;

	let pendingSource: string | null = null;

	function doIncrementalParse(currentSource: string) {
		const currentState = untrack(() => incrementalState);

		try {
			const parseResult = parseIncremental(currentSource, currentState);
			result = parseResult.result;
			incrementalState = parseResult.state;
		} catch (error) {
			console.error('Incremental parse error:', error);
			parseMarkdown(currentSource)
				.then((r) => {
					result = r;
					incrementalState = {
						prevSource: currentSource,
						prevResult: r,
						newContentStart: currentSource.length
					};
				})
				.catch((e) => console.error('Fallback parse error:', e));
		}
	}

	async function doFullParse(currentSource: string) {
		try {
			result = await parseMarkdown(currentSource);
			incrementalState = null;
		} catch (error) {
			console.error('Parse error:', error);
			result = null;
		}
	}

	$effect(() => {
		pendingSource = source;

		if (!incremental) {
			if (throttleTimer != null) {
				clearTimeout(throttleTimer);
				throttleTimer = null;
			}
			doFullParse(source);
		} else {
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
		}
	});
</script>

{#if result == null}
	<div class="space-y-2">
		{#each source.split('\n') as line}
			<p>{line}</p>
		{/each}
	</div>
{:else}
	<div class="space-y-2">
		<Parser nodes={result.nodes} />
	</div>
{/if}
