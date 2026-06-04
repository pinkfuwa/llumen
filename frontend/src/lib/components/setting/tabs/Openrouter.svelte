<script lang="ts">
	import ModelGrid from '$lib/components/setting/ModelGrid.svelte';
	import Warning from '$lib/components/setting/Warning.svelte';
	import { Plus } from '@lucide/svelte';
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
	import Button from '$lib/ui/Button.svelte';

	let { id = $bindable(), value = $bindable() }: { id?: number; value: string } = $props();
</script>

<Warning>{m['setting.config_override_warning'](lang)}</Warning>
<div class="my-2 border-b border-border pb-2">
	<Button
		class="flex w-full flex-row items-center justify-between px-3 py-2"
		onclick={() => (value = 'openrouter_new')}
	>
		{m['setting.add_model'](lang)}
		<Plus class="lucide-icon lucide lucide-trash h-10 w-10 rounded-lg p-2" />
	</Button>
</div>
<ModelGrid bind:id bind:value />
