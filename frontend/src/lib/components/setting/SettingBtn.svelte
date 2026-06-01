<script lang="ts">
	import { Settings } from '@lucide/svelte';
	import { Dialog } from 'bits-ui';
	import { getCurrentUser } from '$lib/api/user.svelte.js';

	const btnStyle =
		'flex w-full items-center justify-between rounded-lg border border-border px-3 py-2 text-center text-sm font-medium text-foreground duration-150 hover:bg-interactive-hover focus:ring-4 focus:ring-ring focus:outline-none';

	let { value = $bindable() } = $props();

	const data = $derived(getCurrentUser());
</script>

<Dialog.Trigger onclick={() => (value = 'account')} class={btnStyle}>
	<span class="font-medium"
		>{#if data == undefined}
			loading{:else}{data!.username}{/if}</span
	>
	<Settings />
</Dialog.Trigger>
