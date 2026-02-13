<script lang="ts">
	import McpGrid from './mcp/McpGrid.svelte';
	import { Plus } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import Button from '$lib/ui/Button.svelte';

	let { id = $bindable(), value = $bindable() }: { id?: number; value: string } = $props();
</script>

<div class="my-2 border-b border-outline pb-2">
	<Button
		class="flex w-full flex-row items-center justify-between px-3 py-2"
		onclick={() => (value = 'mcp_new')}
	>
		{$_('setting.add_mcp_server')}
		<Plus class="lucide-icon lucide lucide-trash h-10 w-10 rounded-lg p-2" />
	</Button>
</div>
<McpGrid bind:id bind:value />
