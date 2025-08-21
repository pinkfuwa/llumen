<script lang="ts">
	import katex from 'katex';

	let { raw } = $props<{ raw: string }>();

	let text = raw.replace(/^\$\$\s*/gm, '').replace(/\s*\$\$$/g, '');

	let rawHTML = $derived(
		katex.renderToString('\\newcommand\\abs[1]{\\lvert#1\\rvert}' + text, {
			displayMode: true,
			output: 'mathml',
			throwOnError: false
		})
	);
</script>

<div class="overflow-x-auto rounded-md px-2 py-[7px] font-mono">
	{@html rawHTML}
</div>
