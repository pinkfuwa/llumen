<script lang="ts">
	import '../app.css';
	import { theme, locale, token } from '$lib/store';
	import { isLoading } from 'svelte-i18n';
	import { setLocale } from '$lib/i18n';
	import { setTheme } from '$lib/theme';
	import { RenewToken } from '$lib/api/user';
	import ErrorMessage from '$lib/components/ErrorMessage.svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { CreateSseInternal } from '$lib/sse';
	import { useError } from '$lib/error';

	let { children } = $props();

	const guardPrefix = ['/chat', '/setting'];

	$effect(() => {
		return token.subscribe((token) => {
			const pathname = page.url.pathname;
			if (token == undefined && guardPrefix.some((m) => pathname.startsWith(m))) {
				goto(`/login?callback=${encodeURIComponent(pathname)}`);
			}
		});
	});

	const error = useError();
	$effect(() => {
		error.subscribe((error) => {
			if (error?.error == 'malformed_token') token.set(undefined);
		});
	});

	CreateSseInternal();
	theme.subscribe(setTheme);
	locale.subscribe(setLocale);
	token.subscribe((data) => {
		if (data) {
			const expireAt = new Date(data.expireAt);
			const renewAt = new Date(data.renewAt);
			const now = new Date();
			const timeout = renewAt.getTime() - now.getTime();
			if (expireAt < now) {
				token.set(undefined);
			} else if (timeout > 0) {
				const timeoutId = setTimeout(() => RenewToken(data.value), timeout);
				return () => clearTimeout(timeoutId);
			} else {
				RenewToken(data.value);
			}
		}
	});
</script>

{#if !$isLoading}
	<div class="h-full w-full bg-light text-dark">
		{@render children()}
		<ErrorMessage />
	</div>
{/if}
