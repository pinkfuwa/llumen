<script lang="ts">
	import { useUser } from '$lib/api/user';
	import { Settings } from '@lucide/svelte';
	import Button from '$lib/ui/Button.svelte';
	import { Dialog } from 'bits-ui';

	let { data, isLoading } = useUser();
</script>

<Dialog.Trigger
	class="flex w-full items-center justify-between rounded-lg border border-outline px-3 py-2 text-center text-sm font-medium text-text duration-150 hover:bg-primary hover:text-text-hover focus:ring-4 focus:ring-outline focus:outline-none  "
>
	<span class="font-medium"
		>{#if $isLoading}
			loading{:else}{$data!.username}{/if}</span
	>
	<Settings />
</Dialog.Trigger>
