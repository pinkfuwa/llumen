<script lang="ts">
	import katex from 'katex';

	let { raw } = $props<{ raw: string }>();

	let text = raw.replace(/^\$\s*/gm, '').replace(/\s*\$*$/g, '');

	let rawHTML = $derived(
		katex.renderToString('\\newcommand\\abs[1]{\\lvert#1\\rvert}' + text, {
			displayMode: false,
			output: 'mathml',
			throwOnError: false
		})
	);
</script>

<span class="rounded-md p-2">
	{@html rawHTML}
</span>
