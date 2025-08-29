<script lang="ts">
	import { goto } from '$app/navigation';
	import { createModel, defaultModelConfig } from '$lib/api/model';
	import { TiltBtn } from '$lib/components';
	import ConfigEditor from '$lib/components/setting/ConfigEditor.svelte';
	import { _ } from 'svelte-i18n';

	let config = $state(defaultModelConfig);

	let { mutate } = createModel();
</script>

<ConfigEditor bind:value={config}>
	<TiltBtn
		class="rounded-lg border border-outline bg-light px-5 py-2 text-dark shadow-sm hover:bg-hover"
		onclick={() => mutate({ config }, () => goto('/setting/openrouter'))}
	>
		{$_('setting.create_setting')}
	</TiltBtn>
</ConfigEditor>
