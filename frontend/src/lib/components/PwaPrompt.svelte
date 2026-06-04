<script lang="ts">
	import Button from '$lib/ui/Button.svelte';
	// @ts-expect-error - virtual module has no type declarations
	import { useRegisterSW } from 'virtual:pwa-register/svelte';
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
	const { offlineReady, needRefresh, updateServiceWorker } = useRegisterSW({
		onRegisteredSW(swUrl: string, r: ServiceWorkerRegistration | undefined) {
			r &&
				setInterval(async () => {
					if (r.installing || !navigator) return;

					if ('connection' in navigator && !navigator.onLine) return;

					const resp = await fetch(swUrl, {
						cache: 'no-store',
						headers: {
							cache: 'no-store',
							'cache-control': 'no-cache'
						}
					});

					if (resp?.status === 200) await r.update();
				}, 20000 /* 20s for testing purposes */);
		},
		onRegisterError(error: Error) {
			console.log('SW registration error', error);
		}
	});
	const close = () => {
		offlineReady.set(false);
		needRefresh.set(false);
	};

	const toast = $derived($offlineReady || $needRefresh);

	const promptStyle =
		'fixed right-0 bottom-0 m-4 rounded border border-border bg-input p-3 shadow-sm';
</script>

{#if toast}
	<div class={promptStyle}>
		<div class="pb-2 text-foreground">
			{#if $offlineReady}
				<span> {m['pwa.offline_ready'](lang)} </span>
			{:else}
				<span> {m['pwa.update_available'](lang)} </span>
			{/if}
		</div>
		{#if $needRefresh}
			<Button onclick={() => updateServiceWorker(true)}>{m['pwa.reload'](lang)}</Button>
		{/if}
		<Button onclick={close}>{m['pwa.close'](lang)}</Button>
	</div>
{/if}
