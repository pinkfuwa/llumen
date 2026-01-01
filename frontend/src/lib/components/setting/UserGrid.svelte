<script lang="ts">
	import { deleteUser, useUsersQueryEffect, getUsers, getCurrentUser } from '$lib/api/user.svelte';
	import { _ } from 'svelte-i18n';
	import CheckDelete from './CheckDelete.svelte';

	const { mutate: deleteUserMutation } = deleteUser();
	useUsersQueryEffect();

	const data = $derived(getUsers());
	const userData = $derived(getCurrentUser());
</script>

{#if data == undefined}
	<div class="mb-4 flex items-center justify-center p-6 text-lg">Loading users...</div>
{:else}
	<div>
		<ul class="grid w-full grid-cols-1 gap-2 overflow-y-auto pb-2 text-lg xl:grid-cols-2">
			{#each data.list as user}
				<li
					class="flex min-h-12 shrink-0 items-center justify-between rounded-lg border border-outline py-1 pr-2 pl-4"
				>
					{user.name}
					{#if userData != undefined && user.id != userData?.user_id}
						<CheckDelete
							ondelete={() =>
								deleteUserMutation({
									user_id: user.id
								})}
						/>
					{/if}
				</li>
			{/each}
		</ul>
	</div>
{/if}
