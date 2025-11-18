<script lang="ts">
	import 'temml/dist/Temml-Latin-Modern.css';

	import { toHtml } from './latex';

	let { text, displayMode = false } = $props<{ text: string; displayMode?: boolean }>();
	let html = $state<string | null>(null);

	// FIXME: this take a assumption that toHtml finish in order
	// look for self-activating signal when modifing this function
	$effect(() => {
		toHtml(text, displayMode).then((res) => (html = res));
	});
</script>

<span>
	{#if html != null}
		{@html html}
	{/if}
</span>
