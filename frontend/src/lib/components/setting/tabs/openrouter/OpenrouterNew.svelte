<script lang="ts">
	import { createModel, defaultModelConfig } from '$lib/api/model.svelte';
	import ConfigEditor from '$lib/components/setting/ConfigEditor.svelte';
	import Button from '$lib/ui/Button.svelte';
	import { t } from 'svelte-intl-precompile';

	let { value = $bindable() }: { value: string } = $props();
	let config = $state(defaultModelConfig);

	async function onCreate() {
		const result = await createModel({ config });
		if (result === 'success') value = 'openrouter';
	}
</script>

<ConfigEditor bind:value={config}>
	<Button class="px-3 py-2" onclick={onCreate}>
		{$t('setting.create_setting')}
	</Button>
</ConfigEditor>
