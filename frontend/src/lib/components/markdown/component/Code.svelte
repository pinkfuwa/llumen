<script lang="ts">
	import type { CodeBlockNode } from '../parser/types';
	import { ClipboardCopy } from '@lucide/svelte';
	import { copy } from '$lib/copy';
	import ShikiCode from '../../shiki/Code.svelte';
	import Mermaid from '../../mermaid/Mermaid.svelte';
	import { isMermaidLanguage } from '../../mermaid/mermaid';
	import Button from '$lib/ui/Button.svelte';

	let { node }: { node: CodeBlockNode } = $props();

	const language = $derived(node.language || 'text');
	const content = $derived(node.content);
	const shouldHighlight = $derived(node.closed);
	const isMermaid = $derived(isMermaidLanguage(language));
</script>

<div class="group/codeblock relative">
	{#if content.split('\n').length >= 2}
		<Button
			borderless
			class=" absolute top-1 right-1 p-2 group-hover/codeblock:visible md:invisible"
			onclick={() => copy(content)}
		>
			<ClipboardCopy class="h-6 w-6" />
		</Button>
	{/if}
	{#if isMermaid}
		<Mermaid text={content} closed={shouldHighlight} />
	{:else}
		<ShikiCode text={content} lang={language} monochrome={!shouldHighlight} />
	{/if}
</div>
