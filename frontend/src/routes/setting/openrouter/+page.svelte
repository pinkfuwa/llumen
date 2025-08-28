<script lang="ts">
	import ModelGrid from '$lib/components/setting/ModelGrid.svelte';
	import Warning from '$lib/components/setting/Warning.svelte';
	import { Plus } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
</script>

<Warning message={$_('setting.config_override_warning')} />
<div class="my-2 border-b border-outline pb-2">
	<a
		class="flex min-h-[50px] w-full shrink-0 items-center justify-between rounded-lg border border-outline py-1 pr-2 pl-4 text-lg"
		href="/setting/openrouter/new"
	>
		Add New Model
		<Plus />
	</a>
</div>

<ModelGrid />
