<script lang="ts">
	import type {
		Token,
		CitationToken,
		TextToken,
		ImageToken,
		LinkToken,
		TableCellToken,
		LatexBlockToken,
		LatexInlineToken,
		InlineCodeToken,
		CodeBlockToken
	} from './lexer';
	import { TokenType } from './lexer';

	import Blockquote from './Blockquote.svelte';
	import Br from './Br.svelte';
	import Citation from './Citation.svelte';
	import Code from './Code.svelte';
	import Codespan from './Codespan.svelte';
	import Del from './Del.svelte';
	import Heading from './Heading.svelte';
	import Hr from './Hr.svelte';
	import Image from './Image.svelte';
	import Italic from './Italic.svelte';
	import Latex from './Latex.svelte';
	import LatexSpan from './LatexSpan.svelte';
	import Link from './Link.svelte';
	import List from './List.svelte';
	import ListItem from './ListItem.svelte';
	import Paragraph from './Paragraph.svelte';
	import Strong from './Strong.svelte';
	import Table from './Table.svelte';
	import TableRow from './TableRow.svelte';
	import TableCell from './TableCell.svelte';
	import Text from './Text.svelte';
	import Parser from './Parser.svelte';

	let { tokens, source, monochrome }: { tokens: Token[]; source: string; monochrome: boolean } =
		$props();
</script>

{#each tokens as token}
	{#if token.type === TokenType.Heading}
		<Heading {token}>
			<Parser tokens={token.children || []} {source} {monochrome} />
		</Heading>
	{:else if token.type === TokenType.Paragraph}
		<Paragraph>
			<Parser tokens={token.children || []} {source} {monochrome} />
		</Paragraph>
	{:else if token.type === TokenType.CodeBlock}
		<Code token={token as CodeBlockToken} {monochrome} />
	{:else if token.type === TokenType.Blockquote}
		<Blockquote>
			<Parser tokens={token.children || []} {source} {monochrome} />
		</Blockquote>
	{:else if token.type === TokenType.OrderedList || token.type === TokenType.UnorderedList}
		<List {token} {source}>
			<Parser tokens={token.children || []} {source} {monochrome} />
		</List>
	{:else if token.type === TokenType.ListItem}
		<ListItem>
			<Parser tokens={token.children || []} {source} {monochrome} />
		</ListItem>
	{:else if token.type === TokenType.Table}
		<Table>
			<Parser tokens={token.children || []} {source} {monochrome} />
		</Table>
	{:else if token.type === TokenType.TableRow}
		<TableRow>
			<Parser tokens={token.children || []} {source} {monochrome} />
		</TableRow>
	{:else if token.type === TokenType.TableCell || token.type === TokenType.TableHeader}
		<TableCell token={token as TableCellToken}>
			<Parser tokens={token.children || []} {source} {monochrome} />
		</TableCell>
	{:else if token.type === TokenType.HorizontalRule}
		<Hr />
	{:else if token.type === TokenType.LatexBlock}
		<Latex token={token as LatexBlockToken} />
	{:else if token.type === TokenType.LatexInline}
		<LatexSpan token={token as LatexInlineToken} />
	{:else if token.type === TokenType.Bold}
		<Strong>
			<Parser tokens={token.children || []} {source} {monochrome} />
		</Strong>
	{:else if token.type === TokenType.Italic}
		<Italic>
			<Parser tokens={token.children || []} {source} {monochrome} />
		</Italic>
	{:else if token.type === TokenType.Strikethrough}
		<Del {token} {source}>
			<Parser tokens={token.children || []} {source} {monochrome} />
		</Del>
	{:else if token.type === TokenType.InlineCode}
		<Codespan token={token as InlineCodeToken} />
	{:else if token.type === TokenType.Link}
		<Link token={token as LinkToken}>
			<Parser tokens={token.children || []} {source} {monochrome} />
		</Link>
	{:else if token.type === TokenType.Image}
		<Image token={token as ImageToken} />
	{:else if token.type === TokenType.Citation}
		{@const citToken = token as CitationToken}
		<Citation
			raw={citToken.title || `[@${citToken.id}]`}
			title={citToken.title || `Citation ${citToken.id}`}
			url={citToken.url || ''}
			favicon={citToken.favicon || ''}
			authoritative={citToken.authoritative || false}
		/>
	{:else if token.type === TokenType.LineBreak}
		<Br />
	{:else if token.type === TokenType.Text}
		<Text token={token as TextToken} />
	{:else}
		<span>Unknown token type: {token.type}</span>
	{/if}
{/each}
