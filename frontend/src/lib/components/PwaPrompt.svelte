<script lang="ts">
	import Button from '$lib/ui/Button.svelte';
	// @ts-expect-error - virtual module has no type declarations
	import { useRegisterSW } from 'virtual:pwa-register/svelte';
	import { _ } from 'svelte-i18n';
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
</script>

{#if toast}
	<div
		class="fixed right-0 bottom-0 m-4 rounded border border-outline bg-input p-3 shadow-login-bg"
	>
		<div class="pb-2 text-text">
			{#if $offlineReady}
				<span> {$_('pwa.offline_ready')} </span>
			{:else}
				<span> {$_('pwa.update_available')} </span>
			{/if}
		</div>
		{#if $needRefresh}
			<Button onclick={() => updateServiceWorker(true)}>{$_('pwa.reload')}</Button>
		{/if}
		<Button onclick={close}>{$_('pwa.close')}</Button>
	</div>
{/if}
