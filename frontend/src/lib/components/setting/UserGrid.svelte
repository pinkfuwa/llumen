<script lang="ts">
	import { useUser, useUsers } from '$lib/api/user';
	import { Trash } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';

	const { isLoading, data } = useUsers();

	const { data: userData } = useUser();
</script>

{#if $isLoading}
	<div class="mb-4 flex items-center justify-center p-6 text-lg">Loading users...</div>
{:else if $data != undefined}
	<ul class="grid max-h-[50vh] grid-cols-1 gap-2 overflow-y-auto pb-2 text-lg lg:grid-cols-2">
		{#each $data as user}
			<li
				class="flex min-h-[50px] items-center justify-between rounded-lg border border-outline py-1 pr-2 pl-4"
			>
				{user.username}
				<!-- TODO: expose delete button if not current user -->
				<!-- {#if user.id != $userData?.user_id}
					<button onclick={() => {}}>
						<Trash class="h-10 w-10 rounded-lg p-2 hover:bg-hover" />
					</button>
				{/if} -->
			</li>
		{/each}
	</ul>
{/if}
