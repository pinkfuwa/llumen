<script lang="ts">
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
	import { Collapsible } from 'bits-ui';
	import { ChevronDown } from '@lucide/svelte';
	import UserGrid from '../UserGrid.svelte';
	import Button from '$lib/ui/Button.svelte';
	import Warning from '../Warning.svelte';
	import { createUser } from '$lib/api/user.svelte';
	import type { MutationStatus } from '$lib/api';

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
			<span>{m['setting.admin.create'](lang)}</span>
			<ChevronDown />
		</Collapsible.Trigger>
		<Collapsible.Content
			class="flex flex-col border-b border-border px-2 slide-out-to-start-0 slide-in-from-top-0 fade-in fade-out data-[state=close]:animate-out data-[state=open]:animate-in"
		>
			{#if status === 'failed'}
				<Warning>{m['setting.admin.error_creating_user'](lang)}</Warning>
			{/if}
			{#if success}
				<div class="mb-2 text-center text-sm">
					{m['setting.admin.user'](lang)} <span class="rounded-md bg-muted p-1">{username}</span>
					{m['setting.admin.created'](lang)}
				</div>
			{/if}
			<div class="mb-2 flex flex-col gap-2">
				<input
					type="text"
					class="w-full rounded-md border border-border p-2"
					placeholder={m['setting.username'](lang)}
					bind:value={username}
					oninput={() => (success = false)}
				/>
				<input
					type="password"
					class="w-full rounded-md border border-border p-2"
					placeholder={m['setting.account.password'](lang)}
					bind:value={password}
					oninput={() => (success = false)}
				/>
				<input
					type="password"
					class="w-full rounded-md border border-border p-2"
					placeholder={m['setting.account.confirm_password'](lang)}
					bind:value={passwordCheck}
					oninput={() => (success = false)}
				/>
			</div>
			<Button
				class="mb-2 w-full {canSubmit ? '' : 'opacity-60'}"
				disabled={!canSubmit}
				onclick={handleSubmit}
			>
				{m['setting.admin.create'](lang)}
			</Button>
		</Collapsible.Content>
	</Collapsible.Root>

	<div class="mb-4 hidden space-y-2 border-b border-border pb-2 text-lg md:block">
		<div class="text-lg">
			{m['setting.admin.create'](lang)}
		</div>
		<div class="flex flex-row items-center justify-between">
			{m['setting.username'](lang)}
			<input
				type="text"
				class="max-w-80 grow rounded-md border border-border p-1"
				placeholder={m['setting.username'](lang)}
				bind:value={username}
				oninput={() => (success = false)}
			/>
		</div>
		<div class="flex flex-row items-center justify-between">
			{m['setting.account.password'](lang)}
			<input
				type="password"
				class="max-w-80 grow rounded-md border border-border p-1"
				placeholder={m['setting.account.password'](lang)}
				bind:value={password}
				oninput={() => (success = false)}
			/>
		</div>
		<div class="flex flex-row items-center justify-between">
			{m['setting.account.confirm_password'](lang)}
			<input
				type="password"
				class="max-w-80 grow rounded-md border border-border p-1"
				placeholder={m['setting.account.confirm_password'](lang)}
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
				{m['setting.admin.create'](lang)}
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
			<h3 class="text-lg">{m['setting.admin.users'](lang)}</h3>
		</div>
		<Collapsible.Root class="md:hidden">
			<Collapsible.Trigger
				class="flex w-full flex-row flex-nowrap justify-between rounded p-2 text-lg duration-150 hover:bg-interactive-hover"
			>
				<h3>{m['setting.admin.users'](lang)}</h3>
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
