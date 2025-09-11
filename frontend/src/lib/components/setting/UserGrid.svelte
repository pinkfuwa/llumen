<script lang="ts">
	import { DeleteUser, useUser, useUsers } from '$lib/api/user';
	import { Trash } from '@lucide/svelte';
	import { Tooltip } from '@svelte-plugins/tooltips';
	import { _ } from 'svelte-i18n';
	import CheckDelete from './CheckDelete.svelte';

	const { mutate: deleteUser } = DeleteUser();
	const { isLoading, data } = useUsers();

	const { isLoading: isUserDataLoading, data: userData } = useUser();
</script>

{#if $isLoading}
	<div class="mb-4 flex items-center justify-center p-6 text-lg">Loading users...</div>
{:else if $data != undefined}
	<ul class="grid max-h-[50vh] grid-cols-1 gap-2 overflow-y-auto pb-2 text-lg lg:grid-cols-2">
		{#each $data.list as user}
			<li
				class="flex min-h-[50px] shrink-0 items-center justify-between rounded-lg border border-outline py-1 pr-2 pl-4"
			>
				{user.name}
				{#if !$isUserDataLoading && user.id != $userData?.user_id}
					<CheckDelete
						ondelete={() =>
							deleteUser({
								user_id: user.id
							})}
					/>
				{/if}
			</li>
		{/each}
	</ul>
{/if}
