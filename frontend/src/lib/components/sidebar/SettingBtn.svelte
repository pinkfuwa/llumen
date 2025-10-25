<script lang="ts">
	import { Settings } from '@lucide/svelte';
	import { Dialog } from 'bits-ui';
	import type { Readable } from 'svelte/store';
	import type { UserReadResp } from '$lib/api/types';
	import { getContext } from 'svelte';

	let { value = $bindable() } = $props();

	const data = getContext<Readable<UserReadResp | undefined>>('user');
</script>

<Dialog.Trigger
	onclick={() => (value = 'account')}
	class="flex w-full cursor-pointer items-center justify-between rounded-lg border border-outline px-3 py-2 text-center
	text-sm font-medium text-text duration-150 hover:bg-primary hover:text-text-hover focus:ring-4 focus:ring-outline focus:outline-none"
>
	<span class="font-medium"
		>{#if $data == undefined}
			loading{:else}{$data!.username}{/if}</span
	>
	<Settings />
</Dialog.Trigger>
