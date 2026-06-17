<script lang="ts">
	import { getThemeStyle } from './shiki';
	import { preference } from '$lib/preference/index.svelte';
	import Monochrome from './Monochrome.svelte';
	import Incremental from './Incremental.svelte';
	import Static from './Static.svelte';
	import { untrack } from 'svelte';

	let {
		lang = 'text',
		text = '',
		incremental = false
	}: { lang?: string; text?: string; incremental?: boolean } = $props();

	// svelte-ignore state_referenced_locally
	let everIncremental = $state(incremental);

	$effect(() => {
		void incremental;
		untrack(() => {
			everIncremental = everIncremental || incremental;
		});
	});

	let themeStyle = $derived(getThemeStyle(preference.value.theme));
</script>

{#if text.trim().length != 0}
	<div
		class="border-radius-md overflow-x-auto rounded-md border border-border p-2"
		style={themeStyle}
	>
		{#if lang === 'text'}
			<Monochrome {text} />
		{:else if incremental}
			<Incremental {text} {lang} />
		{:else}
			<Static {text} {lang} />
		{/if}
	</div>
{/if}
