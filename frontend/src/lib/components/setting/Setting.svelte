<script lang="ts">
	import { _ } from 'svelte-i18n';
	import { fade } from 'svelte/transition';
	import { Star, X } from '@lucide/svelte';
	import { CircleUser, EthernetPort, LogOut, ShieldUser } from '@lucide/svelte';
	import { token } from '$lib/store';
	import { goto } from '$app/navigation';
	import { clearCache } from '$lib/api/state';
	import { Dialog, Label, Separator, Tabs } from 'bits-ui';
	import SettingBtn from '../sidebar/SettingBtn.svelte';
	import Account from './tabs/Account.svelte';
	import Admin from './tabs/Admin.svelte';
	import Openrouter from './tabs/Openrouter.svelte';
	import OpenrouterNew from './tabs/openrouter/OpenrouterNew.svelte';
	import OpenrouterEdit from './tabs/openrouter/OpenrouterEdit.svelte';

	let { open = $bindable() } = $props();
	let value = $state('account');
	let id: undefined | number = $state(undefined);

	$effect(() => {
		if ($token == undefined) goto('/login');
	});
</script>

<Dialog.Root>
	<SettingBtn bind:value />
	<Dialog.Portal>
		<Dialog.Overlay
			class="fixed inset-0 z-50 backdrop-blur-md fade-in-100 fade-out-0 data-[state=closed]:animate-out data-[state=open]:animate-in"
		/>
		<Dialog.Content
			class="md:6/7 fixed inset-0 z-50 m-auto flex rounded-xl border border-outline bg-popup-bg p-3 font-mono fade-in fade-out zoom-in zoom-out data-[state=closed]:animate-out data-[state=open]:animate-in md:w-5/7 lg:h-5/7 lg:w-3/5 xl:h-5/6"
		>
			<Dialog.Close
				class="focus-visible:ring-foreground absolute top-5 right-5 rounded-md focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-background focus-visible:outline-hidden active:scale-[0.98]"
			>
				<div>
					<X class="text-foreground size-5" />
					<span class="sr-only">Close</span>
				</div>
			</Dialog.Close>

			<Tabs.Root bind:value class="flex w-full flex-row">
				<Tabs.List class="flex w-70 flex-col space-y-2 border-r-2 border-outline pr-2 text-xl">
					<Tabs.Trigger
						value="account"
						class="rounded px-3 py-2 text-left duration-150 hover:bg-primary hover:text-text-hover data-[state=active]:bg-primary data-[state=active]:text-text-hover"
					>
						<CircleUser class="mr-2 inline-block h-5 w-5" />
						{$_('setting.account_settings')}
					</Tabs.Trigger>
					<Tabs.Trigger
						value="admin"
						class="rounded px-3 py-2 text-left duration-150 hover:bg-primary hover:text-text-hover data-[state=active]:bg-primary data-[state=active]:text-text-hover"
					>
						<ShieldUser class="mr-2 inline-block h-5 w-5" />
						{$_('setting.admin_settings')}
					</Tabs.Trigger>
					<Tabs.Trigger
						value="openrouter"
						class="rounded px-3 py-2 text-left duration-150 hover:bg-primary hover:text-text-hover data-[state=active]:bg-primary data-[state=active]:text-text-hover"
					>
						<EthernetPort class="mr-2 inline-block h-5 w-5" /> Openrouter
					</Tabs.Trigger>
					<button
						class="rounded px-3 py-2 text-left duration-150 hover:bg-primary hover:text-text-hover"
						onclick={() => {
							token.set(undefined);
							clearCache();
							goto('/login');
						}}
					>
						<LogOut class="mr-2 inline-block h-5 w-5" />
						{$_('setting.logout')}
					</button>
					<a
						class="rounded px-3 py-2 text-left duration-150 hover:bg-primary hover:text-text-hover"
						href="https://github.com/pinkfuwa/llumen"
						target="_blank"
					>
						<Star class="mr-2 inline-block h-5 w-5" /> {$_('setting.github_star')}</a
					>
				</Tabs.List>
				<div class="flex w-full min-w-0 flex-1 justify-center p-3">
					<Tabs.Content value="account">
						<Dialog.Title class="pb-6 text-center text-xl">
							{$_('setting.account_settings')}
						</Dialog.Title>
						<Account />
					</Tabs.Content>
					<Tabs.Content value="admin">
						<Dialog.Title class="pb-6 text-center text-xl">
							{$_('setting.admin_settings')}
						</Dialog.Title>
						<Admin />
					</Tabs.Content>
					<Tabs.Content value="openrouter">
						<Dialog.Title class="pb-6 text-center text-xl">Openrouter</Dialog.Title>
						<Openrouter bind:id bind:value />
					</Tabs.Content>
					<Tabs.Content
						value="openrouter_new"
						class="flex w-full flex-col justify-between overflow-auto"
					>
						<Dialog.Title class="pb-6 text-center text-xl">
							{$_('setting.add_model')}
						</Dialog.Title>

						<OpenrouterNew bind:value />
					</Tabs.Content>
					<Tabs.Content
						value="openrouter_edit"
						class="flex w-full flex-col justify-between overflow-auto"
					>
						<Dialog.Title class="pb-6 text-center text-xl">
							{$_('setting.edit_model')}
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
