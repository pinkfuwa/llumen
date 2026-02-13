<script lang="ts">
	let { id, value = $bindable() }: { id: number; value: string } = $props();

	import { readMcpServerConfig, updateMcpServer } from '$lib/api/mcp.svelte';
	import McpConfigEditor from './McpConfigEditor.svelte';
	import Button from '$lib/ui/Button.svelte';
	import { _ } from 'svelte-i18n';

	let config = $state('');
	let readPromise = $derived(readMcpServerConfig(id).then((x) => (config = x)));

	let saveSetting = $derived($_('setting.save_settings'));
	let loading = $derived($_('common.loading'));
	let error = $derived($_('common.error'));

	let { mutate } = updateMcpServer();
</script>

{#await readPromise}
	{loading}
{:then _}
	{#key id}
		<McpConfigEditor bind:value={config}>
			<Button
				class="px-3 py-2"
				onclick={() => mutate({ id, config_raw: config }, () => (value = 'mcp'))}
			>
				{saveSetting}
			</Button>
		</McpConfigEditor>
	{/key}
{:catch}
	{error}
{/await}
