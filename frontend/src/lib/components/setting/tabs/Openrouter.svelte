<script lang="ts">
	import ModelGrid from '$lib/components/setting/ModelGrid.svelte';
	import Warning from '$lib/components/setting/Warning.svelte';
	import { Plus } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import Button from '$lib/ui/Button.svelte';

	let { id = $bindable(), value = $bindable() }: { id?: number; value: string } = $props();
</script>

<Warning>{$_('setting.config_override_warning')}</Warning>
<div class="my-2 border-b border-outline pb-2">
	<Button
		class="flex w-full flex-row items-center justify-between px-3 py-2"
		onclick={() => (value = 'openrouter_new')}
	>
		{$_('setting.add_model')}
		<Plus class="lucide-icon lucide lucide-trash h-10 w-10 rounded-lg p-2" />
	</Button>
</div>
<ModelGrid bind:id bind:value />
