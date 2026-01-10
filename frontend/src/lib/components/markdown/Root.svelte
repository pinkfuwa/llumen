<script lang="ts">
	import { untrack } from 'svelte';
	import Parser from './Parser.svelte';
	import { parseMarkdown } from './worker';
	import { parseIncremental, type IncrementalState } from './lexer';
	import type { ParseResult } from './lexer';

	const { source, incremental = false }: { source: string; incremental?: boolean } = $props();

	let incrementalState: IncrementalState | null = $state(null);
	let result: ParseResult | null = $state(null);

	let throttleTimer: ReturnType<typeof setTimeout> | null = null;
	const THROTTLE_MS = 100;

	let pendingSource: string | null = null;

	async function doIncrementalParse(currentSource: string) {
		const currentState = untrack(() => incrementalState);

		try {
			const parseResult = await parseIncremental(currentSource, currentState);
			result = parseResult.result;
			incrementalState = parseResult.state;
		} catch (error) {
			console.error('Incremental parse error:', error);
			// Fallback to full parse
			result = await parseMarkdown(currentSource);
			incrementalState = {
				prevSource: currentSource,
				prevResult: result,
				newContentStart: currentSource.length
			};
		}
	}

	async function doFullParse(currentSource: string) {
		try {
			result = await parseMarkdown(currentSource);
			// Reset incremental state
			incrementalState = null;
		} catch (error) {
			console.error('Parse error:', error);
			// Show raw text on error
			result = null;
		}
	}

	$effect(() => {
		pendingSource = source;

		if (!incremental) {
			// Non-incremental mode: use web worker, no throttling
			if (throttleTimer != null) {
				clearTimeout(throttleTimer);
				throttleTimer = null;
			}
			doFullParse(source);
		} else {
			// Incremental mode: parse in main thread with throttling
			if (!throttleTimer) {
				const runThrottle = async () => {
					const lastParsedSource = untrack(() => pendingSource);
					if (lastParsedSource !== null) {
						await doIncrementalParse(lastParsedSource);
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
		<Parser tokens={result.tokens} {source} />
	</div>
{/if}
