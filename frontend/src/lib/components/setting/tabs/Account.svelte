<script lang="ts">
	import { Tabs } from 'bits-ui';
	import { _ } from 'svelte-i18n';
	import AccountGeneral from './account/AccountGeneral.svelte';
	import AccountPassword from './account/AccountPassword.svelte';
	import Warning from '../Warning.svelte';

	let value = $state('general');
</script>

<Tabs.Root bind:value class="flex w-full flex-col">
	<Tabs.List class="mb-4 flex w-full flex-row justify-around border-b-2 border-outline text-lg">
		<Tabs.Trigger
			value="general"
			class="w-full rounded-t-md px-3 py-2 duration-150 hover:bg-primary hover:text-text-hover data-[state=active]:bg-primary data-[state=active]:text-text-hover"
		>
			{$_('setting.account.general')}
		</Tabs.Trigger>
		<Tabs.Trigger
			value="password"
			class="w-full rounded-t-md px-3 py-2 duration-150 hover:bg-primary hover:text-text-hover data-[state=active]:bg-primary data-[state=active]:text-text-hover"
		>
			{$_('setting.account.password')}
		</Tabs.Trigger>
	</Tabs.List>
	<div class="w-full min-w-0 justify-center">
		<Tabs.Content value="general">
			<AccountGeneral />
		</Tabs.Content>
		<Tabs.Content value="password">
			<AccountPassword />
		</Tabs.Content>
	</div>
</Tabs.Root>
