<script lang="ts">
	import { createModel, defaultModelConfig } from '$lib/api/model.svelte';
	import { TiltBtn } from '$lib/components';
	import ConfigEditor from '$lib/components/setting/ConfigEditor.svelte';
	import Button from '$lib/ui/Button.svelte';
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);

	let { value = $bindable() }: { value: string } = $props();
	let config = $state(defaultModelConfig);

	async function onCreate() {
		const result = await createModel({ config });
		if (result === 'success') value = 'openrouter';
	}
</script>

<ConfigEditor bind:value={config}>
	<Button class="px-3 py-2" onclick={onCreate}>
		{m['setting.create_setting'](lang)}
	</Button>
</ConfigEditor>
