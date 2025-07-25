<script lang="ts">
	import '../app.css';
	import { QueryClientProvider, QueryClient } from '@tanstack/svelte-query';
	import { SvelteQueryDevtools } from '@tanstack/svelte-query-devtools';
	import { setLocale } from '$lib/paraglide/runtime';
	import { language } from '$lib/store';
	import { dev } from '$app/environment';

	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				staleTime: Infinity
			}
		}
	});

	let { children } = $props();

	$effect(() => {
		setLocale(language().current);
	});
</script>

<QueryClientProvider client={queryClient}>
	{@render children()}
	{#if dev}
		<SvelteQueryDevtools />
	{/if}
</QueryClientProvider>
