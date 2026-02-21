<script lang="ts">
	import Button from '$lib/ui/Button.svelte';
	import { useRegisterSW } from 'virtual:pwa-register/svelte';
	const { offlineReady, needRefresh, updateServiceWorker } = useRegisterSW({
		onRegisteredSW(swUrl, r) {
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
		onRegisterError(error) {
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
				<span> App ready to work offline </span>
			{:else}
				<span> New content available, click on reload button to update. </span>
			{/if}
		</div>
		{#if $needRefresh}
			<Button onclick={() => updateServiceWorker(true)}>Reload</Button>
		{/if}
		<Button onclick={close}>Close</Button>
	</div>
{/if}
