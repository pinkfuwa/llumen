<script lang="ts">
	import { syntaxHighlight } from './highlight.svelte';
	let {
		text = '',
		lang = 'text',
		incremental = false
	}: {
		text?: string;
		lang?: string;
		incremental?: boolean;
	} = $props();
	let lines = $derived.by(() => {
		let offset = 0;
		return text.split('\n').map((text) => {
			const line = { text, offset };
			offset += text.length + 1;
			return line;
		});
	});
</script>

<pre class="code-text" {@attach syntaxHighlight(() => ({ text, lang, incremental }))}><code
		>{#each lines as line (line.offset)}<div class="code-line"><span data-code-offset={line.offset}
					>{line.text}</span
				></div>{/each}</code
	></pre>

<style>
	.code-text {
		margin: 0;
		padding: 0;
		font: inherit;
		font-family: var(--font-mono);
		line-height: 1.5rem;
		white-space: pre;
		tab-size: 4;
	}
	.code-line {
		min-height: 1.5rem;
	}
</style>
