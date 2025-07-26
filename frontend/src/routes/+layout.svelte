<script lang="ts">
	import '../app.css';
	import { QueryClientProvider, QueryClient } from '@tanstack/svelte-query';
	import { SvelteQueryDevtools } from '@tanstack/svelte-query-devtools';
	import { setLocale } from '$lib/paraglide/runtime';
	import { useLanguage } from '$lib/store';
	import { dev } from '$app/environment';

	const queryClient = new QueryClient();

	let { children } = $props();

	let language = useLanguage();
	$effect(() => {
		setLocale(language.current);
	});
</script>

<QueryClientProvider client={queryClient}>
	{@render children()}
	{#if dev}
		<SvelteQueryDevtools />
	{/if}
</QueryClientProvider>
