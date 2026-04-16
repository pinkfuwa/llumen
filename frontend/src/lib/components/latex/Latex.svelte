<script lang="ts">
	import 'katex/dist/katex.min.css';
	import { toHtml } from './latex';

	let { text, displayMode = false } = $props<{ text: string; displayMode?: boolean }>();
	let html = $state<string | null>(null);

	$effect(() => {
		let originalText = text;
		toHtml(originalText, displayMode).then((res) => {
			if (text == originalText) html = res;
		});
	});
</script>

{#if html != null}
	{@html html}
{:else}
	{text}
{/if}
