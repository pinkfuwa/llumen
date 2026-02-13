<script lang="ts">
	import { getMcpServers, deleteMcpServer } from '$lib/api/mcp.svelte';
	import { _ } from 'svelte-i18n';
	import CheckDelete from '../../CheckDelete.svelte';
	import Button from '$lib/ui/Button.svelte';

	let { id = $bindable(), value = $bindable() }: { id?: number; value: string } = $props();
	const { mutate: deleteServerMutation } = deleteMcpServer();
	const data = $derived(getMcpServers());
</script>

{#if data == undefined}
	<div class="mb-4 flex items-center justify-center p-6 text-lg">Loading MCP servers...</div>
{:else}
	<div class="grow space-y-2 overflow-y-auto">
		{#each data.list as server (server.id)}
			<Button
				class="flex w-full flex-row items-center justify-between px-3 py-2"
				onclick={() => {
					id = server.id;
					value = 'mcp_edit';
				}}
			>
				<div class="flex flex-col items-start">
					<span>{server.name}</span>
					<span class="text-xs text-text-secondary">
						{server.transport}
						{#if server.running}
							· <span class="text-green-500">running</span>
						{:else}
							· <span class="text-text-secondary">stopped</span>
						{/if}
						{#if !server.enabled}
							· <span class="text-yellow-500">disabled</span>
						{/if}
					</span>
				</div>
				<CheckDelete
					ondelete={() =>
						deleteServerMutation({
							id: server.id
						})}
				/>
			</Button>
		{/each}
		{#if data.list.length === 0}
			<div class="flex items-center justify-center p-6 text-text-secondary">
				{$_('setting.no_mcp_servers')}
			</div>
		{/if}
	</div>
{/if}
