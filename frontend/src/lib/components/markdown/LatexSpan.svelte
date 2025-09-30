<script lang="ts">
	import katex from 'katex';
	import { latexTrim } from './latex';

	let { raw } = $props<{ raw: string }>();

	let text = $derived(latexTrim(raw));

	let rawHTML = $derived(
		katex.renderToString(text, {
			displayMode: false,
			output: 'mathml',
			throwOnError: false
		})
	);
</script>

<span class="rounded-md p-2 font-semibold">
	{@html rawHTML}
</span>
