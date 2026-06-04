<script>
	import { CircleCheck } from '@lucide/svelte';
	import { fade } from 'svelte/transition';
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
	import { copyCounter } from '$lib/copy.svelte';

	let copied = $state(true);

	const hintStyle =
		'fixed top-0 right-0 z-6 m-3 flex items-center rounded-md border border-border bg-popover bg-card px-3 py-2';

	$effect(() => {
		const timeoutId = setTimeout(() => (copied = false), 500);
		return () => clearTimeout(timeoutId);
	});
</script>

{#if copyCounter.val != 0}
	{#key copyCounter.val}
		{#if copied}
			<div class={hintStyle} in:fade={{ duration: 150 }} out:fade={{ duration: 150 }}>
				<CircleCheck class="mr-2 inline-block" />
				{m['common.copied_clipboard'](lang)}
			</div>
		{/if}
	{/key}
{/if}
