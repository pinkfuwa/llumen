<script lang="ts">
	import '../app.css';
	import { theme, locale, token } from '$lib/store';
	import { isLoading } from 'svelte-i18n';
	import { setLocale } from '$lib/i18n';
	import { setTheme } from '$lib/theme';
	import { RenewToken } from '$lib/api/user';

	let { children } = $props();

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
	</div>
{/if}
