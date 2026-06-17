<script lang="ts">
	import { parseSync, createAstRenderer } from './parser/renderer';
	import { parser, parser_write } from './parser/smd';
	import type { AstNode, Parser as SMDParser } from './parser/types';
	import { useThrottle } from '$lib/throttle.svelte';
	import { untrack } from 'svelte';
	import Parser from './Parser.svelte';

	const { source, incremental = false }: { source: string; incremental?: boolean } = $props();

	let nodes: AstNode[] = $state([]);

	let p: SMDParser = parser(createAstRenderer(nodes).renderer);
	let prevLength = 0;

	function resetParser() {
		prevLength = 0;
		nodes.splice(0);
	}

	function doStreamingParse(currentSource: string) {
		try {
			if (currentSource.length < prevLength) {
				untrack(() => resetParser());
			}
			const delta = currentSource.slice(prevLength);
			if (delta.length > 0) {
				untrack(() => parser_write(p!, delta));
				prevLength = currentSource.length;
			}
		} catch (error) {
			console.error('Streaming parse error:', error);
			untrack(() => resetParser());
		}
	}

	function doFullParse(currentSource: string) {
		try {
			untrack(() => resetParser());
			const result = parseSync(currentSource + '\n');

			untrack(() => {
				nodes.splice(0);
				nodes.push(...result);
			});
		} catch (error) {
			console.error('Parse error:', error);
			untrack(() => {
				nodes.splice(0);
			});
		}
	}

	const throttledParse = useThrottle((s: string) => {
		doStreamingParse(s);
	}, 100);

	$effect(() => {
		// finishing cause a full repaint, which is expected.
		// Also markdown parse synchronized, so no flashing to user.
		if (incremental) {
			throttledParse(source);
		} else {
			throttledParse.cancel();
			doFullParse(source);
		}
	});
</script>

<div class="space-y-2">
	<Parser {nodes} />
</div>
