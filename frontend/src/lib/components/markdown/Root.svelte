<script lang="ts">
	import { parseSync, createAstRenderer } from './parser/renderer';
	import { parser, parser_write } from './parser/smd';
	import type { AstNode, Parser as SMDParser } from './parser/types';
	import { useThrottle } from '$lib/throttle.svelte';
	import { untrack } from 'svelte';
	import Parser from './Parser.svelte';
	import { prompt } from '$lib/copy.svelte';
	import { expandSourceRange, getSourceOffset } from './source-utils';

	const {
		source,
		incremental = false,
		class: className,
		copy = false
	}: { source: string; incremental?: boolean; class?: string; copy?: boolean } = $props();

	let nodes: AstNode[] = $state([]);
	let containerRef: HTMLDivElement | undefined = $state();

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

	function handleCopy(e: ClipboardEvent) {
		const selection = window.getSelection();
		if (!selection || selection.isCollapsed) return;
		const range = selection.getRangeAt(0);
		if (!containerRef || !containerRef.contains(range.commonAncestorContainer)) return;

		const startOffset = getSourceOffset(range.startContainer, range.startOffset, containerRef);
		const endOffset = getSourceOffset(range.endContainer, range.endOffset, containerRef);
		if (startOffset == null || endOffset == null) return;

		let s = Math.min(startOffset, endOffset);
		let endPos = Math.max(startOffset, endOffset);
		[s, endPos] = expandSourceRange(source, s, endPos);
		const text = source.slice(s, endPos);
		if (text.length === 0) return;
		e.clipboardData?.setData('text/plain', text);
		e.preventDefault();
		prompt();
	}
</script>

<div
	bind:this={containerRef}
	class={className ? `${className} space-y-2` : 'space-y-2'}
	oncopy={copy ? handleCopy : undefined}
>
	<Parser {nodes} />
</div>
