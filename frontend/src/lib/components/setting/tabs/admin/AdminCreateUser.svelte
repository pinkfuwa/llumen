<script lang="ts">
	import { CheckLine } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import CheckPwd from '../../CheckPwd.svelte';
	import { CreateUser } from '$lib/api/user';

	let func = $state<'general' | 'retypePwd' | 'notify'>('general');
	let username = $state('');

	let { mutate: createUserMutate } = CreateUser();
</script>

{#if func == 'notify'}
	<div class="font-semibold">
		{$_('setting.admin.user')} <span class="bg-hover rounded-md p-2">{username}</span>
		{$_('setting.admin.created')}
	</div>
{:else if func == 'retypePwd'}
	<CheckPwd
		message={$_('setting.admin.type_password_for_user', { values: { username } })}
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
		<div class="grow">
			<label for="name">{$_('setting.create_user')}: </label>
			<input
				type="text"
				id="name"
				class="w-full rounded-md border border-outline p-1"
				bind:value={username}
				placeholder={$_('setting.username')}
			/>
		</div>

		<button
			class="hover:bg-hover mx-1 mr-2 shrink-0 rounded-md p-1"
			onclick={() => {
				if (username.length != 0) func = 'retypePwd';
			}}><CheckLine /></button
		>
	</div>
{/if}
