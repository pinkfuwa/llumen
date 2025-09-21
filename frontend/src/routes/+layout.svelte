<script lang="ts">
	import '../app.css';
	import { isLoading } from 'svelte-i18n';
	import ErrorMessage from '$lib/components/ErrorMessage.svelte';
	import { initError, useError } from '$lib/error';
	import { copyCounter } from '$lib/copy';
	import CopyHint from '$lib/components/buttons/CopyHint.svelte';
	import initLatex from '$lib/components/markdown/latex';
	import { initPreference } from '$lib';
	import { initAuth } from '$lib/api/auth';
	import initCitation from '$lib/components/markdown/citation';
	import initMap from '$lib/components/markdown/map';

	let { children } = $props();

	initError();
	initAuth();

	initPreference();
	initLatex();
	initCitation();
	initMap();
</script>

{#if !$isLoading}
	<div class="h-full w-full bg-white text-black dark:bg-gray-700">
		{@render children()}
		<ErrorMessage />
		{#if $copyCounter != 0}
			{#key $copyCounter}
				<CopyHint />
			{/key}
		{/if}
	</div>
{/if}
