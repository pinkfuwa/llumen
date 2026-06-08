<script>
	import { CircleCheck } from '@lucide/svelte';
	import { fade } from 'svelte/transition';
	import { copyCounter } from '$lib/copy.svelte';
	import { t } from 'svelte-intl-precompile';

	let copied = $state(false);

	$effect(() => {
		if (copyCounter.val == 0) return;
		copied = true;
		const timeoutId = setTimeout(() => (copied = false), 500);
		return () => clearTimeout(timeoutId);
	});
</script>

{#if copyCounter.val != 0}
	{#if copied}
		<div
			class="fixed top-0 right-0 z-6 m-3 flex items-center rounded-md border border-border bg-card px-3 py-2 text-popover-foreground"
			in:fade={{ duration: 150 }}
			out:fade={{ duration: 150 }}
			onmouseleave={() => {
				copied = false;
			}}
			role="tooltip"
		>
			<CircleCheck class="mr-2 inline-block" />
			{$t('common.copied_clipboard')}
		</div>
	{/if}
{/if}
