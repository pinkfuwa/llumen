<script lang="ts">
	import Parser from './Parser.svelte';
	import { parseSync, createAstRenderer } from './parser/renderer';
	import { parser, parser_write } from './parser/smd';
	import type { AstNode } from './parser/types';
	import { useThrottle } from '$lib/throttle.svelte';
	import { untrack } from 'svelte';

	const { source, incremental = false }: { source: string; incremental?: boolean } = $props();

	let rootChildren: AstNode[] = $state([]);
	let nodes = $derived(rootChildren);

	// it's nullable because parsing rendering is bounded to rootChildren
	let p: ReturnType<typeof parser> | null = null;
	let prevLength = 0;

	function ensureParser() {
		if (p) return;
		const { renderer } = createAstRenderer(rootChildren);
		p = parser(renderer);
		prevLength = 0;
	}

	function resetParser() {
		p = null;
		prevLength = 0;
		rootChildren.splice(0);
	}

	function doStreamingParse(currentSource: string) {
		try {
			if (currentSource.length < prevLength) {
				untrack(() => resetParser());
			}
			untrack(() => ensureParser());
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
				rootChildren = result;
			});
		} catch (error) {
			console.error('Parse error:', error);
			untrack(() => {
				rootChildren = [];
			});
		}
	}

	const throttledParse = useThrottle((s: string) => {
		doStreamingParse(s);
	}, 100);

	$effect(() => {
		if (incremental) {
			throttledParse(source);
		} else {
			// todo: use parse_end to switch incremental parse
			throttledParse.cancel();
			doFullParse(source);
		}
	});
</script>

<div class="space-y-2">
	<Parser {nodes} />
</div>
