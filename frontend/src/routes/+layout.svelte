<script lang="ts">
	import '../app.css';
	import { theme, locale } from '$lib/store';
	import { isLoading } from 'svelte-i18n';
	import { setLocale } from '$lib/i18n';
	import { setTheme } from '$lib/theme';

	let { children } = $props();

	$effect(() => theme.subscribe(setTheme));
	$effect(() => locale.subscribe(setLocale));
</script>

{#if !$isLoading}
	<div class="h-full w-full bg-light text-dark">
		{@render children()}
	</div>
{/if}
