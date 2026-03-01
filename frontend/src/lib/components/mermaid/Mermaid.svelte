<script lang="ts">
	import { render } from './mermaid';
	import { isLightTheme } from '$lib/preference';
	import Code from '../shiki/Code.svelte';

	let { text = '', closed = false } = $props<{ text?: string; closed?: boolean }>();

	let svg = $state<string | null>(null);
	let error = $state<string | null>(null);
	let rendering = $state(false);

	const isDark = $derived(!$isLightTheme);

	$effect(() => {
		if (!closed || !text) {
			svg = null;
			error = null;
			return;
		}

		rendering = true;
		error = null;

		render(text)
			.then((result) => {
				svg = result;
			})
			.catch((e) => {
				error = e.message;
			})
			.finally(() => {
				rendering = false;
			});
	});
</script>

{#if !closed || rendering}
	<Code {text} />
{:else if error}
	<Code {text} />
{:else if svg}
	<div
		class="mermaid border-radius-md flex justify-center overflow-x-auto rounded-md border border-outline bg-white p-4 {isDark
			? 'dark:bg-neutral-900'
			: ''}"
	>
		{@html svg}
	</div>
{/if}
