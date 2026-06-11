<script lang="ts">
	import Parser from './Parser.svelte';
	import { parseSync } from './parser/renderer';
	import { parser, parser_write, parser_end } from './parser/smd';
	import { createAstRenderer } from './parser/renderer';
	import type { AstNode } from './parser/types';
	import { useThrottle } from '$lib/throttle.svelte';

	const { source, incremental = false }: { source: string; incremental?: boolean } = $props();

	let children: { children: AstNode[] } = $state({ children: [] });
	let nodes = $derived(children.children);

	function doStreamingParse(currentSource: string) {
		try {
			const renderer = createAstRenderer();
			const p = parser(renderer.renderer);
			parser_write(p, currentSource);
			parser_end(p);
			children.children = renderer.getResult();
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

	const throttledParse = useThrottle((s: string) => {
		doStreamingParse(s);
	}, 100);

	$effect(() => {
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
