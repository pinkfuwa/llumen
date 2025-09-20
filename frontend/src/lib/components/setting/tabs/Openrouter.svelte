<script lang="ts">
	import ModelGrid from '$lib/components/setting/ModelGrid.svelte';
	import Warning from '$lib/components/setting/Warning.svelte';
	import { Plus } from '@lucide/svelte';
	import { Tabs } from 'bits-ui';
	import { _ } from 'svelte-i18n';
	import OpenrouterNew from './openrouter/OpenrouterNew.svelte';
	let value = $state('');
</script>

<Tabs.Root bind:value>
	<Tabs.Content value="">
		<Warning>{$_('setting.config_override_warning')}</Warning>
		<div class="my-2 border-b border-outline pb-2">
			<Tabs.Trigger
				value="new"
				class="
					flex
					w-full
					flex-row
					items-center
					justify-between
					rounded-lg
					border
					border-outline
					px-3
					py-2
					text-center
					text-text
					duration-150
					hover:bg-primary
					hover:text-text-hover
					focus:ring-4
					focus:ring-outline
					focus:outline-none"
			>
				{$_('setting.add_model')}
				<Plus class="lucide-icon lucide lucide-trash h-10 w-10 rounded-lg p-2" />
			</Tabs.Trigger>
		</div>

		<ModelGrid />
	</Tabs.Content>
	<Tabs.Content value="new">
		<OpenrouterNew bind:value/>
	</Tabs.Content>
</Tabs.Root>
