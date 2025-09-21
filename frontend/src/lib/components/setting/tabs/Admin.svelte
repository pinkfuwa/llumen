<script lang="ts">
	import { Tabs } from 'bits-ui';
	import { _ } from 'svelte-i18n';
	import UserGrid from '../UserGrid.svelte';
	import AdminCreateUser from './admin/AdminCreateUser.svelte';

	let value = $state('users');
</script>

<Tabs.Root bind:value class="flex w-full flex-col">
	<Tabs.List class="mb-4 flex w-full flex-row justify-around border-b-2 border-outline text-lg">
		<Tabs.Trigger
			value="users"
			class="w-full rounded-t-md px-3 py-2 duration-150 hover:bg-primary hover:text-text-hover data-[state=active]:bg-primary data-[state=active]:text-text-hover"
		>
			{$_('setting.admin.users')}
		</Tabs.Trigger>
		<Tabs.Trigger
			value="create"
			class="w-full rounded-t-md px-3 py-2 duration-150 hover:bg-primary hover:text-text-hover data-[state=active]:bg-primary data-[state=active]:text-text-hover"
		>
			{$_('setting.admin.create')}
		</Tabs.Trigger>
	</Tabs.List>
	<div class="w-full min-w-0 justify-center">
		<Tabs.Content value="users">
			<UserGrid />
		</Tabs.Content>
		<Tabs.Content value="create">
			<AdminCreateUser />
		</Tabs.Content>
	</div>
</Tabs.Root>
