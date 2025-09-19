<script lang="ts">
	import ModelGrid from '$lib/components/setting/ModelGrid.svelte';
	import Warning from '$lib/components/setting/Warning.svelte';
	import { Plus } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
</script>

<Warning>{$_('setting.config_override_warning')}</Warning>
<div class="my-2 border-b border-outline pb-2">
	<a
		class="flex min-h-[50px] w-full shrink-0 items-center justify-between rounded-lg border border-outline py-1 pr-2 pl-4 text-lg hover:bg-hover"
		href="/setting/openrouter/new"
	>
		{$_('setting.add_model')}
		<Plus class="lucide-icon lucide lucide-trash h-10 w-10 rounded-lg p-2" />
	</a>
</div>

<ModelGrid />
