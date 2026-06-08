<script lang="ts">
	import { Star, X } from '@lucide/svelte';
	import { CircleUser, EthernetPort, LogOut, ShieldUser } from '@lucide/svelte';
	import { token } from '$lib/rune.svelte';
	import { Dialog, Tabs } from 'bits-ui';
	import SettingBtn from './SettingBtn.svelte';
	import Account from './tabs/Account.svelte';
	import Admin from './tabs/Admin.svelte';
	import Openrouter from './tabs/Openrouter.svelte';
	import OpenrouterNew from './tabs/openrouter/OpenrouterNew.svelte';
	import OpenrouterEdit from './tabs/openrouter/OpenrouterEdit.svelte';
	import { t } from 'svelte-intl-precompile';

	const dialogStyle =
		'fixed inset-0 z-20 m-2 flex max-w-4xl rounded-xl border border-border bg-popover p-3 font-mono text-foreground fade-in fade-out zoom-in zoom-out data-[state=closed]:animate-out data-[state=open]:animate-in md:m-auto md:h-[min(80vh,48rem)] md:w-full';
	const tabStyle =
		'cursor-pointer rounded px-3 py-2 text-left duration-150 hover:bg-interactive-hover data-[state=active]:bg-interactive-selection data-[state=active]:text-primary';
	const linkStyle = 'rounded px-3 py-2 text-left duration-150 hover:bg-interactive-hover';
	const btnRowStyle =
		'cursor-pointer rounded px-3 py-2 text-left duration-150 hover:bg-interactive-hover';

	let { open = $bindable() } = $props();
	let value = $state('account');
	let id: undefined | number = $state(undefined);
</script>

<Dialog.Root>
	<SettingBtn bind:value />
	<Dialog.Portal>
		<Dialog.Overlay
			class="fixed inset-0 z-20 backdrop-blur-md fade-in-100 fade-out-0 data-[state=closed]:animate-out data-[state=open]:animate-in"
		/>
		<Dialog.Content class={dialogStyle}>
			<Dialog.Close
				class="absolute top-5 right-5 rounded-md focus-visible:ring-2 focus-visible:ring-foreground focus-visible:ring-offset-2 focus-visible:ring-offset-background focus-visible:outline-hidden active:scale-[0.98]"
			>
				<div class="pb-3 pl-3">
					<X class="size-5 text-foreground" />
					<span class="sr-only">Close</span>
				</div>
			</Dialog.Close>

			<Tabs.Root bind:value class="flex w-full flex-row">
				<Tabs.List
					class="flex flex-col justify-between border-r-2 border-border pr-2 text-xl md:w-70"
				>
					<div class="flex flex-col space-y-2">
						<Tabs.Trigger value="account" class={tabStyle}>
							<CircleUser class="inline-block h-5 w-5 md:mr-2" />
							<span class="hidden md:inline-block">
								{$t('setting.account_settings')}
							</span>
						</Tabs.Trigger>
						<Tabs.Trigger value="admin" class={tabStyle}>
							<ShieldUser class="inline-block h-5 w-5 md:mr-2" />
							<span class="hidden md:inline-block">
								{$t('setting.admin_settings')}
							</span>
						</Tabs.Trigger>
						<Tabs.Trigger value="openrouter" class={tabStyle}>
							<EthernetPort class="inline-block h-5 w-5 md:mr-2" />
							<span class="hidden md:inline-block"> Openrouter </span>
						</Tabs.Trigger>
					</div>
					<div class="flex flex-col space-y-2">
						<a class={linkStyle} href="https://github.com/pinkfuwa/llumen" target="_blank">
							<Star class="inline-block h-5 w-5 md:mr-2" />
							<span class="hidden md:inline-block"> {$t('setting.github_star')} </span>
						</a>
						<button
							class={btnRowStyle}
							onclick={() => {
								token.value = undefined;
							}}
						>
							<LogOut class="inline-block h-5 w-5 md:mr-2" />
							<span class="hidden md:inline-block"> {$t('setting.logout')} </span>
						</button>
					</div>
				</Tabs.List>
				<div class="flex h-full w-full min-w-0 flex-1 flex-col justify-center overflow-hidden p-3">
					<Tabs.Content value="account" class="flex h-full flex-col overflow-auto">
						<Dialog.Title class="pb-6 text-center text-xl">
							{$t('setting.account_settings')}
						</Dialog.Title>
						<Account />
					</Tabs.Content>
					<Tabs.Content value="admin" class="flex h-full flex-col overflow-auto">
						<Dialog.Title class="pb-6 text-center text-xl">
							{$t('setting.admin_settings')}
						</Dialog.Title>
						<Admin />
					</Tabs.Content>
					<Tabs.Content value="openrouter" class="flex h-full flex-col">
						<Dialog.Title class="pb-6 text-center text-xl">Openrouter</Dialog.Title>
						<Openrouter bind:id bind:value />
					</Tabs.Content>
					<Tabs.Content
						value="openrouter_new"
						class="flex w-full flex-col justify-between overflow-auto"
					>
						<Dialog.Title class="pb-6 text-center text-xl">
							{$t('setting.add_model')}
						</Dialog.Title>

						<OpenrouterNew bind:value />
					</Tabs.Content>
					<Tabs.Content
						value="openrouter_edit"
						class="flex w-full flex-col justify-between overflow-auto"
					>
						<Dialog.Title class="pb-6 text-center text-xl">
							{$t('setting.edit_model')}
						</Dialog.Title>

						{#if id != undefined}
							<OpenrouterEdit {id} bind:value />
						{/if}
					</Tabs.Content>
				</div>
			</Tabs.Root>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
