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
		const children = node.children as { text: string; type: string }[];

		// Find indices of first and second CodeMark
		let firstIndex = -1;
		let secondIndex = -1;
		for (let i = 0; i < children.length; i++) {
			if (children[i].type === 'CodeMark') {
				if (firstIndex === -1) {
					firstIndex = i;
				} else if (secondIndex === -1) {
					secondIndex = i;
					break; // No need to look further after finding the second
				}
			}
		}

		// If no first CodeMark, return empty string
		if (firstIndex === -1) {
			return '';
		}

		// Determine the range for extracting text
		const start = firstIndex + 1;
		const end = secondIndex === -1 ? children.length : secondIndex;

		// Extract text from text nodes in the range
		let result = '';
		for (let i = start; i < end; i++) {
			if (children[i].type === 'CodeText') {
				result += children[i].text;
			}
		}

		return result;
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
