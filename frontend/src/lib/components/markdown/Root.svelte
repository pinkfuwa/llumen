<script lang="ts">
	import { untrack } from 'svelte';
	import Parser from './Parser.svelte';
	import { parseSync } from './vendor/renderer';
	import { parser, parser_write, parser_end } from './vendor/smd';
	import { createAstRenderer } from './vendor/renderer';
	import type { AstNode } from './vendor/types';

	const { source, incremental = false }: { source: string; incremental?: boolean } = $props();

	let children: { children: AstNode[] } = $state({ children: [] });
	let nodes = $derived(children.children);

	let throttleTimer: ReturnType<typeof setTimeout> | null = null;
	const THROTTLE_MS = 100;

	let pendingSource: string | null = null;

	let parserInstance: ReturnType<typeof parser> | null = null;
	let astRenderer: ReturnType<typeof createAstRenderer> | null = null;

	function initStreamingParser() {
		astRenderer = createAstRenderer();
		parserInstance = parser(astRenderer.renderer);
	}

	function doStreamingParse(currentSource: string) {
		if (!parserInstance || !astRenderer) {
			initStreamingParser();
		}
		try {
			parser_write(parserInstance!, currentSource);
			children.children = astRenderer!.getResult();
		} catch (error) {
			console.error('Streaming parse error:', error);
			children.children = [];
		}
	}

	function doFullParse(currentSource: string) {
		try {
			children.children = parseSync(currentSource + '\n');
		} catch (error) {
			console.error('Parse error:', error);
			children.children = [];
		}
	}

	$effect(() => {
		pendingSource = source;

		if (incremental) {
			if (!throttleTimer) {
				initStreamingParser();

				const runThrottle = () => {
					const lastParsedSource = untrack(() => pendingSource);
					if (lastParsedSource !== null) {
						doStreamingParse(lastParsedSource);
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
			parserInstance = null;
			astRenderer = null;

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
