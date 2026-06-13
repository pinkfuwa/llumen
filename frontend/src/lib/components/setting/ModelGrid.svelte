<script lang="ts">
	import { deleteModel, models } from '$lib/api/model.svelte';
	import CheckDelete from './CheckDelete.svelte';
	import Button from '$lib/ui/Button.svelte';

	let { id = $bindable(), value = $bindable() }: { id?: number; value: string } = $props();
	const data = $derived(models.val);
</script>

{#if data == undefined}
	<div class="mb-4 flex items-center justify-center p-6 text-lg">Loading models...</div>
{:else}
	<div class="grow space-y-2 overflow-y-auto">
		{#each data as model (model.id)}
			<Button
				class="flex w-full flex-row items-center justify-between px-3 py-2"
				onclick={() => {
					id = model.id;
					value = 'openrouter_edit';
				}}
			>
				{model.display_name}
				<CheckDelete ondelete={() => deleteModel({ id: model.id })} />
			</Button>
		{/each}
	</div>
{/if}
