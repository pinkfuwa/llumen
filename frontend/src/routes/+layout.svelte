<script lang="ts">
	import '../app.css';
	import { QueryClientProvider, QueryClient } from '@tanstack/svelte-query';
	import { SvelteQueryDevtools } from '@tanstack/svelte-query-devtools';
	import { setLocale } from '$lib/paraglide/runtime';
	import { useLanguage, useTheme } from '$lib/store';
	import { dev } from '$app/environment';
	import { getThemeStyle } from '$lib/theme';

	const queryClient = new QueryClient();

	let { children } = $props();

	let language = useLanguage();
	$effect(() => {
		setLocale($language);
	});

	let theme = useTheme();
</script>

<QueryClientProvider client={queryClient}>
	<div class="h-full w-full bg-light text-dark" style={getThemeStyle($theme)}>
		{@render children()}
	</div>
	{#if dev}
		<SvelteQueryDevtools />
	{/if}
</QueryClientProvider>
