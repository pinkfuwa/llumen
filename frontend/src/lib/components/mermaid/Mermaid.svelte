<script lang="ts">
	import { render } from './mermaid';
	import { isLightTheme } from '$lib/preference';

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
	<div
		class="border-radius-md flex min-h-[3rem] w-full items-center justify-center overflow-x-auto rounded-md border border-outline bg-neutral-50 px-4 py-2 text-sm text-neutral-600 {isDark
			? 'dark:bg-neutral-900 dark:text-neutral-400'
			: ''}"
	>
		<pre class="font-mono whitespace-pre-wrap">{text}</pre>
	</div>
{:else if error}
	<div
		class="border-radius-md flex min-h-[3rem] w-full items-center justify-center overflow-x-auto rounded-md border border-red-500 bg-red-50 px-4 py-2 text-sm text-red-600 {isDark
			? 'dark:bg-red-900/30 dark:text-red-400'
			: ''}"
	>
		<pre class="font-mono whitespace-pre-wrap">{text}</pre>
	</div>
{:else if svg}
	<div class="mermaid border-radius-md flex justify-center overflow-x-auto rounded-md border border-outline bg-white p-4 {isDark
			? 'dark:bg-neutral-900'
			: ''}"
	>
		{@html svg}
	</div>
{/if}
