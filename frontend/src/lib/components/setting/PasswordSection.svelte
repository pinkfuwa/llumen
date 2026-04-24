<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { Collapsible } from 'bits-ui';
	import { ChevronDown, CheckLine } from '@lucide/svelte';
	import Button from '$lib/ui/Button.svelte';
	import Warning from './Warning.svelte';
	import { updateUser } from '$lib/api/user.svelte';
	import { token } from '$lib/store';

	let { mutate, isError } = updateUser();

	let open = $state(false);
	let password = $state('');
	let passwordCheck = $state('');
	let bounceKey = $state(0);

	let matched = $derived(password.length != 0 && password == passwordCheck);

	function handleSubmit() {
		if (matched) {
			mutate({ password }, () => {
				token.set(undefined);
			});
		} else {
			bounceKey += 1;
			passwordCheck = '';
		}
	}
</script>

<div class="mb-4 hidden items-center justify-between border-b border-outline pb-2 text-lg md:flex">
	<span>{$_('setting.change_password')}: </span>
	<div class="flex items-center gap-2">
		<input
			type="password"
			class="w-36 rounded-md border border-outline p-1"
			placeholder={$_('setting.account.password')}
			bind:value={password}
		/>
		<input
			type="password"
			class="w-36 rounded-md border border-outline p-1"
			placeholder={$_('setting.account.confirm_password')}
			bind:value={passwordCheck}
		/>
		<Button class="p-2 {matched ? '' : 'opacity-60'}" disabled={!matched} onclick={handleSubmit}>
			<CheckLine class="size-5" />
		</Button>
	</div>
</div>

<Collapsible.Root bind:open class="md:hidden">
	<Collapsible.Trigger
		class="flex w-full flex-row flex-nowrap justify-between rounded p-2 text-lg duration-150 hover:bg-primary hover:text-text-hover"
	>
		<span>{$_('setting.change_password')}</span>
		<ChevronDown />
	</Collapsible.Trigger>
	<Collapsible.Content
		class="flex flex-col border-b border-outline px-2 slide-out-to-start-0 slide-in-from-top-0 fade-in fade-out data-[state=close]:animate-out data-[state=open]:animate-in"
	>
		{#if isError()}
			<Warning>{$_('setting.account.error_updating_password')}</Warning>
		{/if}
		<div class="mb-2 flex flex-col gap-2">
			<input
				type="password"
				class="w-full rounded-md border border-outline p-2"
				placeholder={$_('setting.account.password')}
				bind:value={password}
			/>
			<input
				type="password"
				class="w-full rounded-md border border-outline p-2"
				placeholder={$_('setting.account.confirm_password')}
				bind:value={passwordCheck}
			/>
		</div>
		<Button
			class="mb-2 w-full {matched ? '' : 'opacity-60'}"
			disabled={!matched}
			onclick={handleSubmit}
		>
			{$_('setting.confirm')}
		</Button>
	</Collapsible.Content>
</Collapsible.Root>

{#if bounceKey != 0}
	{#key bounceKey}
		<div class="hidden md:block">
			<Warning>TODO: fill the translation</Warning>
		</div>
	{/key}
{/if}
