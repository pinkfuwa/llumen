<script lang="ts">
	import { GetUsers, CreateUser } from '$lib/api/user';
	import { Trash, CheckLine } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { goto } from '$app/navigation';
	import { useToken } from '$lib/store';

	let token = useToken();
	let username = $state('');
	let password = $state('');

	let createdUser = $state('');

	const usersQuery = GetUsers(() => token.current || '');
</script>

{#if createdUser.length != 0}
	<div class="mb-2 rounded-lg bg-red-700 hover:bg-red-500">
		<div class="ml-2 p-3 font-semibold hover:bg-hover">
			user <span class="rounded-md bg-hover p-2">{createdUser}</span> created
		</div>
	</div>
{/if}

<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
	<label for="name">{m.create_user()}: </label>
	<div class="flex items-center justify-between">
		<input
			type="text"
			id="name"
			class="rounded-md border border-outline p-1"
			bind:value={username}
			placeholder={m.username()}
		/>
		<button
			class="mx-1 rounded-md p-1 hover:bg-hover"
			onclick={() => {
				if (username.length != 0) goto('/setting/admin/create/' + encodeURIComponent(username));
			}}><CheckLine /></button
		>
	</div>
</div>

{#if $usersQuery.isPending}
	<div class="mb-4 flex items-center justify-center border-b border-outline p-6 text-lg">
		Loading users...
	</div>
{/if}
{#if $usersQuery.isSuccess}
	<ul
		class="grid grid-cols-1 gap-2 border-b border-outline pb-2 text-lg lg:grid-cols-2 2xl:grid-cols-3"
	>
		{#each $usersQuery.data.users as user}
			<li class="flex items-center justify-between rounded-lg border border-outline py-1 pr-2 pl-4">
				{user.username}
				<Trash class="h-10 w-10 rounded-lg p-2 hover:bg-hover" />
			</li>
		{/each}
	</ul>
{/if}
