<script lang="ts">
	import '../app.css';
	import { isLoading } from 'svelte-i18n';
	import ErrorMessage from '$lib/components/ErrorMessage.svelte';
	import { copyCounter } from '$lib/copy';
	import CopyHint from '$lib/components/common/CopyHint.svelte';
	import '$lib/preference/index.svelte';

	let { children } = $props();
</script>

{#await import('$lib/components/PwaPrompt.svelte') then { default: ReloadPrompt }}
	<ReloadPrompt />
{/await}

{#if !$isLoading}
	<div class="bg-surface-base h-full w-full text-foreground">
		{@render children()}
		<ErrorMessage />
		{#if $copyCounter != 0}
			{#key $copyCounter}
				<CopyHint />
			{/key}
		{/if}
	</div>
{/if}
