<script lang="ts">
	import { untrack } from 'svelte';
	import Parser from './Parser.svelte';
	import { parseMarkdown, parseMarkdownIncremental, walkTree } from './lezer';

	const { source, monochrome = false, incremental = false } = $props();

	let prevSource: string = $state('');
	let prevTree: import('@lezer/common').Tree | null = null;
	let ast: any = $state(null);

	$effect(() => {
		const prevTreeValue = untrack(() => prevTree);
		const prevSourceValue = untrack(() => prevSource);
		let tree: import('@lezer/common').Tree;

		if (
			incremental &&
			prevTreeValue &&
			source.startsWith(prevSourceValue) &&
			source.length > prevSourceValue.length
		) {
			tree = parseMarkdownIncremental(prevTreeValue, prevSourceValue, source);
		} else {
			tree = parseMarkdown(source);
		}

		prevSource = source;
		prevTree = tree;
		ast = walkTree(tree, source);
	});
</script>

<div class="space-y-2">
	<Parser {ast} {monochrome} />
</div>
