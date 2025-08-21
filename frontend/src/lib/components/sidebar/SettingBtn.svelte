<script lang="ts">
	import { useUser } from '$lib/api/user';
	import { Settings } from '@lucide/svelte';

	let { data, isLoading } = useUser();
</script>

<a
	class="flex w-full items-center justify-between rounded-lg border border-outline bg-light px-3 py-2 select-none hover:bg-hover"
	href="/setting/account"
>
	<span class="font-medium text-dark"
		>{#if $isLoading}
			loading{:else}{$data!.username}{/if}</span
	>
	<Settings class="hover:text-dark" />
</a>
