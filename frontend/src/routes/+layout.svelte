<script lang="ts">
	import '../app.css';
	import { isLoading } from 'svelte-i18n';
	import ErrorMessage from '$lib/components/ErrorMessage.svelte';
	import { initError } from '$lib/error';
	import { copyCounter } from '$lib/copy';
	import CopyHint from '$lib/components/buttons/CopyHint.svelte';
	import { initPreference } from '$lib';
	import { initAuth } from '$lib/api/auth';

	let { children } = $props();

	initError();
	initAuth();

	initPreference();
</script>

{#if !$isLoading}
	<div class="h-full w-full bg-white text-text">
		{@render children()}
		<ErrorMessage />
		{#if $copyCounter != 0}
			{#key $copyCounter}
				<CopyHint />
			{/key}
		{/if}
	</div>
{/if}
