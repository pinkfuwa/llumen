<script lang="ts">
	import type { ASTNode, CitationNode } from './lexer/parser';

	import Blockquote from './Blockquote.svelte';
	import Br from './Br.svelte';
	import Citation from './Citation.svelte';
	import Code from './Code.svelte';
	import Codespan from './Codespan.svelte';
	import Del from './Del.svelte';
	import Heading from './Heading.svelte';
	import Hr from './Hr.svelte';
	import Html from './Html.svelte';
	import Image from './Image.svelte';
	import Latex from './Latex.svelte';
	import LatexSpan from './LatexSpan.svelte';
	import Link from './Link.svelte';
	import List from './List.svelte';
	import ListItem from './ListItem.svelte';
	import Paragraph from './Paragraph.svelte';
	import Strong from './Strong.svelte';
	import Table from './Table.svelte';
	import TableBody from './TableBody.svelte';
	import TableCell from './TableCell.svelte';
	import TableHead from './TableHead.svelte';
	import TableRow from './TableRow.svelte';
	import Text from './Text.svelte';
	import Parser from './Parser.svelte';
	import Empty from './Empty.svelte';

	type Segment = { type: 'text'; text: string } | { type: 'node'; node: ASTNode };

	let { ast }: { ast: ASTNode } = $props();

	const nodeMap: Record<string, any> = {
		Blockquote,
		HardBreak: Br,
		Citation,
		CodeBlock: Code,
		FencedCode: Code,
		InlineCode: Codespan,
		Strikethrough: Del,
		StrikethroughMark: Empty,
		Emphasis: Strong,
		ATXHeading1: Heading,
		ATXHeading2: Heading,
		ATXHeading3: Heading,
		ATXHeading4: Heading,
		ATXHeading5: Heading,
		ATXHeading6: Heading,
		SetextHeading1: Heading,
		SetextHeading2: Heading,
		ThematicBreak: Hr,
		HorizontalRule: Hr,
		HTMLBlock: Html,
		HTMLTag: Html,
		Image,
		LatexBlock: Latex,
		LatexSpan,
		InlineMathBracket: LatexSpan,
		InlineMathBracketMark: Empty,
		InlineMathDollar: LatexSpan,
		InlineMathDollarMark: Empty,
		BlockMathBracket: Latex,
		BlockMathBracketMark: Empty,
		BlockMathDollar: Latex,
		BlockMathDollarMark: Empty,
		Link,
		URL: Empty,
		Autolink: Link,
		BulletList: List,
		OrderedList: List,
		ListItem,
		Paragraph,
		StrongEmphasis: Strong,
		Table,
		TableBody,
		TableCell,
		TableHeader: TableHead,
		TableRow,
		Text,
		CodeText: Text,
		LinkLabel: Text,
		EmphasisMark: Empty,
		CodeMark: Text,
		LinkMark: Empty,
		QuoteMark: Empty,
		ListMark: Empty,
		HeaderMark: Empty,
		default: Text,
		TableDelimiter: Empty,
		Escape: Empty
	};

	const segments = $derived.by((): Segment[] => {
		if (!ast) return [];

		const text = ast.text ?? '';
		const children = ast.children ?? [];

		if (children.length === 0) {
			return text ? [{ type: 'text', text }] : [];
		}

		const hasPositions = children.every(
			(c: ASTNode) => typeof c.from === 'number' && typeof c.to === 'number'
		);

		if (!hasPositions || typeof ast.from !== 'number') {
			return children.map((c: ASTNode): Segment => ({ type: 'node', node: c }));
		}

		const sortedChildren = children
			.slice()
			.sort((a: ASTNode, b: ASTNode) => (a.from ?? 0) - (b.from ?? 0));

		const result: Segment[] = [];
		const baseOffset = ast.from;
		let lastEnd = 0;

		for (const child of sortedChildren) {
			const childStart = (child.from ?? 0) - baseOffset;
			const childEnd = (child.to ?? 0) - baseOffset;

			if (childStart > lastEnd) {
				const textSlice = text.slice(lastEnd, childStart);
				if (textSlice) {
					result.push({ type: 'text', text: textSlice });
				}
			}

			result.push({ type: 'node', node: child });
			lastEnd = Math.max(lastEnd, childEnd);
		}

		if (lastEnd < text.length) {
			const textSlice = text.slice(lastEnd);
			if (textSlice) {
				result.push({ type: 'text', text: textSlice });
			}
		}

		return result;
	});
</script>

{#if ast}
	{#if ast.type === 'Document'}
		{#each ast.children ?? [] as child}
			<Parser ast={child} />
		{/each}
	{:else if ast.type === 'Citation'}
		{@const data = (ast as CitationNode).citationData}
		<Citation
			raw={data.raw}
			title={data.title}
			url={data.url}
			favicon={data.favicon}
			authoritative={data.authoritative}
		/>
	{:else if !nodeMap[ast.type]}
		<pre>
<code>Unmapped node type: {ast.type}</code>
<code>{JSON.stringify(ast, null, 2)}</code>
		</pre>
	{:else}
		{@const MappedComponent = nodeMap[ast.type]}
		<MappedComponent node={ast}>
			{#each segments as seg}
				{#if seg.type === 'text'}
					{seg.text}
				{:else if seg.type === 'node'}
					<Parser ast={seg.node} />
				{/if}
			{/each}
		</MappedComponent>
	{/if}
{/if}
