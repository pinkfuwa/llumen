<script lang="ts">
	import katex from 'katex';
	import { latexTrim } from './latex';

	let { raw } = $props<{ raw: string }>();

	let text = $derived(latexTrim(raw));

	let rawHTML = $derived(
		katex.renderToString(text, {
			displayMode: true,
			output: 'mathml',
			throwOnError: false
		})
	);
</script>

<div class="flex justify-center overflow-x-auto">
	<div class="rounded-md border border-outline p-4">
		{@html rawHTML}
	</div>
</div>
