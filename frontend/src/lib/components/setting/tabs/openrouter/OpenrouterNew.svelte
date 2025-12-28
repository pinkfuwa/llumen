<script lang="ts">
	import { goto } from '$app/navigation';
	import { createModel, defaultModelConfig } from '$lib/api/model.svelte';
	import { TiltBtn } from '$lib/components';
	import ConfigEditor from '$lib/components/setting/ConfigEditor.svelte';
	import Button from '$lib/ui/Button.svelte';
	import { _ } from 'svelte-i18n';

	let { value = $bindable() }: { value: string } = $props();
	let config = $state(defaultModelConfig);

	let { mutate } = createModel();
</script>

<ConfigEditor bind:value={config}>
	<Button class="px-3 py-2" onclick={() => mutate({ config }, () => (value = 'openrouter'))}>
		{$_('setting.create_setting')}
	</Button>
</ConfigEditor>
