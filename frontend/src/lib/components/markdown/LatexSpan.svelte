<script lang="ts">
	import katex from 'katex';

	let { raw } = $props<{ raw: string }>();

	let text = raw
		.replace(/^\$\s*/gm, '')
		.replace(/\s*\$*$/gm, '')
		.replace(/^\\\[\s*/gm, '')
		.replace(/\s*\\\]$/gm, '');

	let rawHTML = $derived(
		katex.renderToString(text, {
			displayMode: false,
			output: 'mathml',
			throwOnError: false
		})
	);

	console.log(text);
</script>

<span class="rounded-md p-2 font-semibold">
	{@html rawHTML}
</span>
