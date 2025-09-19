<script lang="ts">
	import { CheckLine } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import UserGrid from '$lib/components/setting/UserGrid.svelte';
	import CheckPwd from '$lib/components/setting/CheckPwd.svelte';
	import { CreateUser } from '$lib/api/user';

	let func = $state<'general' | 'retypePwd' | 'notify'>('general');
	let username = $state('');

	let { mutate: createUserMutate } = CreateUser();
</script>

{#if func == 'notify'}
	<div class="font-semibold">
		user <span class="rounded-md bg-hover p-2">{username}</span> created
	</div>
{:else if func == 'retypePwd'}
	<CheckPwd
		message={`Type password for ${username}`}
		onsubmit={(password) => {
			createUserMutate(
				{
					username,
					password
				},
				() => {
					func = 'notify';
				}
			);
		}}
		oncancal={() => (func = 'general')}
	/>
{:else}
	<div class="mb-4 flex items-center justify-between border-b border-outline pb-2 text-lg">
		<label for="name">{$_('setting.create_user')}: </label>
		<div class="flex items-center justify-between">
			<input
				type="text"
				id="name"
				class="rounded-md border border-outline p-1"
				bind:value={username}
				placeholder={$_('setting.username')}
			/>
			<button
				class="mx-1 rounded-md p-1 hover:bg-hover"
				onclick={() => {
					if (username.length != 0) func = 'retypePwd';
				}}><CheckLine /></button
			>
		</div>
	</div>

	<UserGrid />
{/if}
