<script lang="ts">
	import { createModel, defaultModelConfig } from '$lib/api/model.svelte';
	import { TiltBtn } from '$lib/components';
	import ConfigEditor from '$lib/components/setting/ConfigEditor.svelte';
	import Button from '$lib/ui/Button.svelte';
	import { _ } from 'svelte-i18n';

	let { value = $bindable() }: { value: string } = $props();
	let config = $state(defaultModelConfig);

	async function onCreate() {
		const result = await createModel({ config });
		if (result === 'success') value = 'openrouter';
	}
</script>

<ConfigEditor bind:value={config}>
	<Button class="px-3 py-2" onclick={onCreate}>
		{$_('setting.create_setting')}
	</Button>
</ConfigEditor>
