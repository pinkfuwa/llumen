<script lang="ts">
	import CodeText from './CodeText.svelte';
	import { codeThemeStyle } from './theme';
	let {
		lang = 'text',
		text = '',
		incremental = false
	}: {
		lang?: string;
		text?: string;
		incremental?: boolean;
	} = $props();
</script>

{#if text.length !== 0}
	<div class="overflow-x-auto rounded-md border border-border p-2" style={codeThemeStyle}>
		<CodeText {text} {lang} {incremental} />
	</div>
{/if}
