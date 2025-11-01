<script lang="ts">
	import { ClipboardCopy } from '@lucide/svelte';
	import { copy } from '$lib/copy';
	import Code from '../shiki/Code.svelte';
	import Root from './Root.svelte';
	import Latex from './Latex.svelte';

	let { node, monochrome = false } = $props();

	function extractLanguage(node: any): string {
		const infoChild = node.children?.find((c: any) => c.type === 'CodeInfo');
		if (infoChild) {
			return infoChild.text.trim();
		}
		return '';
	}

	function extractCodeText(node: any): string {
		if (node.type === 'FencedCode' || node.type === 'CodeBlock') {
			const codeTextChild = node.children?.find((c: any) => c.type === 'CodeText');
			if (codeTextChild) {
				return codeTextChild.text;
			}
		}
		return node.text || '';
	}

	const lang = $derived(extractLanguage(node));
	const text = $derived(extractCodeText(node));
</script>

{#if lang == 'markdown'}
	<Root source={text} />
{:else if lang == 'latex'}
	<Latex node={{ text }} />
{:else}
	<div class="group/codeblock relative">
		{#if text.split('\n').length > 1}
			<button
				class="absolute top-0 right-0 m-1 duration-150 group-hover/codeblock:visible md:invisible"
				onclick={() => copy(text)}
			>
				<ClipboardCopy
					class="h-10 w-10 rounded-lg p-2 duration-150 hover:bg-primary hover:text-text-hover"
				/>
			</button>
			<Code {lang} {text} {monochrome} />
		{:else}
			<button onclick={() => copy(text)} class="w-full cursor-pointer">
				<Code {lang} {text} {monochrome} />
			</button>
		{/if}
	</div>
{/if}
