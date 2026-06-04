<script lang="ts">
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
	import { Collapsible } from 'bits-ui';
	import { ChevronDown, CheckLine } from '@lucide/svelte';
	import Button from '$lib/ui/Button.svelte';
	import Warning from './Warning.svelte';
	import { updateUser } from '$lib/api/user.svelte';
	import { token } from '$lib/store.svelte';
	import type { MutationStatus } from '$lib/api';

	const triggerStyle =
		'flex w-full flex-row flex-nowrap justify-between rounded p-2 text-lg duration-150 hover:bg-interactive-hover';

	let open = $state(false);
	let password = $state('');
	let passwordCheck = $state('');
	let bounceKey = $state(0);
	let status = $state<MutationStatus>('untried');

	let matched = $derived(password.length != 0 && password == passwordCheck);

	async function handleSubmit() {
		if (matched) {
			const result = await updateUser({ password });
			status = result;
			if (result === 'success') token.value = undefined;
		} else {
			bounceKey += 1;
			passwordCheck = '';
		}
	}
</script>

<div class="mb-4 hidden items-center justify-between border-b border-border pb-2 text-lg md:flex">
	<span>{m['setting.change_password'](lang)}: </span>
	<div class="flex items-center gap-2">
		<input
			type="password"
			class="w-36 rounded-md border border-border p-1"
			placeholder={m['setting.account.password'](lang)}
			bind:value={password}
		/>
		<input
			type="password"
			class="w-36 rounded-md border border-border p-1"
			placeholder={m['setting.account.confirm_password'](lang)}
			bind:value={passwordCheck}
		/>
		<Button class="p-2 {matched ? '' : 'opacity-60'}" disabled={!matched} onclick={handleSubmit}>
			<CheckLine class="size-5" />
		</Button>
	</div>
</div>

<Collapsible.Root bind:open class="md:hidden">
	<Collapsible.Trigger class={triggerStyle}>
		<span>{m['setting.change_password'](lang)}</span>
		<ChevronDown />
	</Collapsible.Trigger>
	<Collapsible.Content
		class="flex flex-col border-b border-border px-2 slide-out-to-start-0 slide-in-from-top-0 fade-in fade-out data-[state=close]:animate-out data-[state=open]:animate-in"
	>
		{#if status === 'failed'}
			<Warning>{m['setting.account.error_updating_password'](lang)}</Warning>
		{/if}
		<div class="mb-2 flex flex-col gap-2">
			<input
				type="password"
				class="w-full rounded-md border border-border p-2"
				placeholder={m['setting.account.password'](lang)}
				bind:value={password}
			/>
			<input
				type="password"
				class="w-full rounded-md border border-border p-2"
				placeholder={m['setting.account.confirm_password'](lang)}
				bind:value={passwordCheck}
			/>
		</div>
		<Button
			class="mb-2 w-full {matched ? '' : 'opacity-60'}"
			disabled={!matched}
			onclick={handleSubmit}
		>
			{m['setting.confirm'](lang)}
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
