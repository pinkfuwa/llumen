<script lang="ts">
	import katex from 'katex';

	let { raw } = $props<{ raw: string }>();

	let text = raw
		.replace(/^\$\$\s*/gm, '')
		.replace(/\s*\$\$$/g, '')
		.replace(/^\\\[\s*/gm, '')
		.replace(/\s*\\\]$/g, '');

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
