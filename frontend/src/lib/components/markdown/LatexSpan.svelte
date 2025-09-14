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

<span class="rounded-md bg-background px-2 py-[7px] font-mono hover:bg-hover">
	{@html rawHTML}
</span>
