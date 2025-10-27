<script lang="ts">
	import { DeleteUser, useUser, useUsers } from '$lib/api/user';
	import { Trash } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import CheckDelete from './CheckDelete.svelte';
	import { getContext } from 'svelte';
	import type { Readable } from 'svelte/store';
	import type { UserReadResp } from '$lib/api/types';

	const { mutate: deleteUser } = DeleteUser();
	const { isLoading, data } = useUsers();

	const userData = getContext<Readable<UserReadResp | undefined>>('user');
</script>

{#if $isLoading}
	<div class="mb-4 flex items-center justify-center p-6 text-lg">Loading users...</div>
{:else if $data != undefined}
	<div>
		<ul class="grid w-full grid-cols-1 gap-2 overflow-y-auto pb-2 text-lg xl:grid-cols-2">
			{#each $data.list as user}
				<li
					class="flex min-h-[3rem] shrink-0 items-center justify-between rounded-lg border border-outline py-1 pr-2 pl-4"
				>
					{user.name}
					{#if $userData != undefined && user.id != $userData?.user_id}
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
	</div>
{/if}
