<script lang="ts">
	import {
		AstNodeType,
		type AstNode,
		type HeadingNode,
		type CodeBlockNode,
		type OrderedListNode,
		type UnorderedListNode,
		type TableCellNode,
		type LatexBlockNode,
		type LatexInlineNode,
		type InlineCodeNode,
		type LinkNode,
		type ImageNode,
		type TextNode
	} from './parser/types';

	import Blockquote from './component/Blockquote.svelte';
	import Br from './component/Br.svelte';
	import Code from './component/Code.svelte';
	import Codespan from './component/Codespan.svelte';
	import Del from './component/Del.svelte';
	import Heading from './component/Heading.svelte';
	import Hr from './component/Hr.svelte';
	import Image from './component/Image.svelte';
	import Italic from './component/Italic.svelte';
	import Latex from './component/Latex.svelte';
	import LatexSpan from './component/LatexSpan.svelte';
	import Link from './component/Link.svelte';
	import List from './component/List.svelte';
	import ListItem from './component/ListItem.svelte';
	import Paragraph from './component/Paragraph.svelte';
	import Strong from './component/Strong.svelte';
	import Table from './component/Table.svelte';
	import TableRow from './component/TableRow.svelte';
	import TableCell from './component/TableCell.svelte';
	import Text from './component/Text.svelte';
	import Parser from './Parser.svelte';

	let { nodes }: { nodes: AstNode[] } = $props();
</script>

{#each nodes as node (node.type + ':' + node.start + ':' + node.end)}
	{#if node.type === AstNodeType.Heading}
		<Heading node={node as HeadingNode}>
			<Parser nodes={node.children || []} />
		</Heading>
	{:else if node.type === AstNodeType.Paragraph}
		<Paragraph>
			<Parser nodes={node.children || []} />
		</Paragraph>
	{:else if node.type === AstNodeType.CodeBlock}
		<Code node={node as CodeBlockNode} />
	{:else if node.type === AstNodeType.Blockquote}
		<Blockquote>
			<Parser nodes={node.children || []} />
		</Blockquote>
	{:else if node.type === AstNodeType.OrderedList || node.type === AstNodeType.UnorderedList}
		<List node={node as OrderedListNode | UnorderedListNode}>
			<Parser nodes={node.children || []} />
		</List>
	{:else if node.type === AstNodeType.ListItem}
		<ListItem>
			<Parser nodes={node.children || []} />
		</ListItem>
	{:else if node.type === AstNodeType.Table}
		<Table>
			<Parser nodes={node.children || []} />
		</Table>
	{:else if node.type === AstNodeType.TableRow}
		<TableRow>
			<Parser nodes={node.children || []} />
		</TableRow>
	{:else if node.type === AstNodeType.TableCell}
		<TableCell node={node as TableCellNode}>
			<Parser nodes={node.children || []} />
		</TableCell>
	{:else if node.type === AstNodeType.HorizontalRule}
		<Hr />
	{:else if node.type === AstNodeType.LatexBlock}
		<Latex node={node as LatexBlockNode} />
	{:else if node.type === AstNodeType.LatexInline}
		<LatexSpan node={node as LatexInlineNode} />
	{:else if node.type === AstNodeType.Bold}
		<Strong>
			<Parser nodes={node.children || []} />
		</Strong>
	{:else if node.type === AstNodeType.Italic}
		<Italic>
			<Parser nodes={node.children || []} />
		</Italic>
	{:else if node.type === AstNodeType.Strikethrough}
		<Del>
			<Parser nodes={node.children || []} />
		</Del>
	{:else if node.type === AstNodeType.InlineCode}
		<Codespan node={node as InlineCodeNode} />
	{:else if node.type === AstNodeType.Link}
		<Link node={node as LinkNode}>
			<Parser nodes={node.children || []} />
		</Link>
	{:else if node.type === AstNodeType.Image}
		<Image node={node as ImageNode} />
	{:else if node.type === AstNodeType.LineBreak}
		<Br />
	{:else if node.type === AstNodeType.Text}
		<Text node={node as TextNode} />
	{/if}
{/each}
