<script lang="ts">
	import { createMcpServer, defaultMcpConfig } from '$lib/api/mcp.svelte';
	import McpConfigEditor from './McpConfigEditor.svelte';
	import Button from '$lib/ui/Button.svelte';
	import { _ } from 'svelte-i18n';

	let { value = $bindable() }: { value: string } = $props();
	let config = $state(defaultMcpConfig);

	let { mutate } = createMcpServer();
</script>

<McpConfigEditor bind:value={config}>
	<Button class="px-3 py-2" onclick={() => mutate({ config_raw: config }, () => (value = 'mcp'))}>
		{$_('setting.create_setting')}
	</Button>
</McpConfigEditor>
