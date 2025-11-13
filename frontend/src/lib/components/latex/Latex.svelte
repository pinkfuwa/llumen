<script lang="ts">
	import { toHtml } from './latex';

	let { text, displayMode = false } = $props<{ text: string; displayMode?: boolean }>();
	let html = $state<string | null>(null);
	$effect(() => {
		toHtml(text, displayMode).then((res) => (html = res));
	});
</script>

{#if html != null}
	{@html html}
{/if}
