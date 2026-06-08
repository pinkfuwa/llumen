<script lang="ts">
	import { Collapsible } from 'bits-ui';
	import { ChevronDown } from '@lucide/svelte';
	import UserGrid from '../UserGrid.svelte';
	import Button from '$lib/ui/Button.svelte';
	import Warning from '../Warning.svelte';
	import { createUser } from '$lib/api/user.svelte';
	import type { MutationStatus } from '$lib/api';
	import { t } from 'svelte-intl-precompile';

	let username = $state('');
	let password = $state('');
	let passwordCheck = $state('');
	let bounceKey = $state(0);
	let success = $state(false);
	let status = $state<MutationStatus>('untried');

	let matched = $derived(password.length != 0 && password == passwordCheck);
	let canSubmit = $derived(username.length != 0 && matched);

	async function handleSubmit() {
		if (canSubmit) {
			const result = await createUser({ username, password });
			status = result;
			if (result === 'success') {
				success = true;
				username = '';
				password = '';
				passwordCheck = '';
			}
		} else {
			bounceKey += 1;
			passwordCheck = '';
		}
	}
</script>

<div class="flex h-full flex-col gap-2 overflow-auto">
	<Collapsible.Root class="md:hidden">
		<Collapsible.Trigger
			class="flex w-full flex-row flex-nowrap justify-between rounded p-2 text-lg duration-150 hover:bg-interactive-hover"
		>
			<span>{$t('setting.admin.create')}</span>
			<ChevronDown />
		</Collapsible.Trigger>
		<Collapsible.Content
			class="flex flex-col border-b border-border px-2 slide-out-to-start-0 slide-in-from-top-0 fade-in fade-out data-[state=close]:animate-out data-[state=open]:animate-in"
		>
			{#if status === 'failed'}
				<Warning>{$t('setting.admin.error_creating_user')}</Warning>
			{/if}
			{#if success}
				<div class="mb-2 text-center text-sm">
					{$t('setting.admin.user')} <span class="rounded-md bg-muted p-1">{username}</span>
					{$t('setting.admin.created')}
				</div>
			{/if}
			<div class="mb-2 flex flex-col gap-2">
				<input
					type="text"
					class="w-full rounded-md border border-border p-2"
					placeholder={$t('setting.username')}
					bind:value={username}
					oninput={() => (success = false)}
				/>
				<input
					type="password"
					class="w-full rounded-md border border-border p-2"
					placeholder={$t('setting.account.password')}
					bind:value={password}
					oninput={() => (success = false)}
				/>
				<input
					type="password"
					class="w-full rounded-md border border-border p-2"
					placeholder={$t('setting.account.confirm_password')}
					bind:value={passwordCheck}
					oninput={() => (success = false)}
				/>
			</div>
			<Button
				class="mb-2 w-full {canSubmit ? '' : 'opacity-60'}"
				disabled={!canSubmit}
				onclick={handleSubmit}
			>
				{$t('setting.admin.create')}
			</Button>
		</Collapsible.Content>
	</Collapsible.Root>

	<div class="mb-4 hidden space-y-2 border-b border-border pb-2 text-lg md:block">
		<div class="text-lg">
			{$t('setting.admin.create')}
		</div>
		<div class="flex flex-row items-center justify-between">
			{$t('setting.username')}
			<input
				type="text"
				class="max-w-80 grow rounded-md border border-border p-1"
				placeholder={$t('setting.username')}
				bind:value={username}
				oninput={() => (success = false)}
			/>
		</div>
		<div class="flex flex-row items-center justify-between">
			{$t('setting.account.password')}
			<input
				type="password"
				class="max-w-80 grow rounded-md border border-border p-1"
				placeholder={$t('setting.account.password')}
				bind:value={password}
				oninput={() => (success = false)}
			/>
		</div>
		<div class="flex flex-row items-center justify-between">
			{$t('setting.account.confirm_password')}
			<input
				type="password"
				class="max-w-80 grow rounded-md border border-border p-1"
				placeholder={$t('setting.account.confirm_password')}
				bind:value={passwordCheck}
				oninput={() => (success = false)}
			/>
		</div>

		<div class="flex justify-center">
			<Button
				class="max-w-80 grow p-2 {canSubmit ? '' : 'opacity-60'}"
				disabled={!canSubmit}
				onclick={handleSubmit}
			>
				{$t('setting.admin.create')}
			</Button>
		</div>
	</div>

	{#if bounceKey != 0}
		{#key bounceKey}
			<div class="hidden md:block">
				<Warning>TODO: fill the translation</Warning>
			</div>
		{/key}
	{/if}

	<div class="flex flex-col">
		<div class="hidden items-center justify-between pb-2 md:flex">
			<h3 class="text-lg">{$t('setting.admin.users')}</h3>
		</div>
		<Collapsible.Root class="md:hidden">
			<Collapsible.Trigger
				class="flex w-full flex-row flex-nowrap justify-between rounded p-2 text-lg duration-150 hover:bg-interactive-hover"
			>
				<h3>{$t('setting.admin.users')}</h3>
				<ChevronDown />
			</Collapsible.Trigger>
			<Collapsible.Content
				class="slide-out-to-start-0 slide-in-from-top-0 fade-in fade-out data-[state=close]:animate-out data-[state=open]:animate-in"
			>
				<UserGrid />
			</Collapsible.Content>
		</Collapsible.Root>
		<div class="hidden md:block"><UserGrid /></div>
	</div>
</div>
