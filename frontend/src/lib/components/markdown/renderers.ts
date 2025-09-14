import Blockquote from './Blockquote.svelte';
import Br from './Br.svelte';
import Code from './Code.svelte';
import Codespan from './Codespan.svelte';
import Del from './Del.svelte';
import Em from './Em.svelte';
import Heading from './Heading.svelte';
import Hr from './Hr.svelte';
import Html from './Html.svelte';
import Image from './Image.svelte';
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
import LatexSpan from './LatexSpan.svelte';
import Latex from './Latex.svelte';
import Citation from './Citation.svelte';

export const renderers = {
	heading: Heading,
	paragraph: Paragraph,
	text: Text,
	image: Image,
	link: Link,
	em: Em,
	strong: Strong,
	codespan: Codespan,
	del: Del,
	table: Table,
	tablehead: TableHead,
	tablebody: TableBody,
	tablerow: TableRow,
	tablecell: TableCell,
	list: List,
	orderedlistitem: null,
	unorderedlistitem: null,
	listitem: ListItem,
	hr: Hr,
	html: Html,
	blockquote: Blockquote,
	code: Code,
	br: Br,
	inlineKatex: LatexSpan,
	blockKatex: Latex,
	citation: Citation
};
