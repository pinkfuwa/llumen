<script lang="ts">
	import { DeleteModel, useModels } from '$lib/api/model';
	import { _ } from 'svelte-i18n';
	import CheckDelete from './CheckDelete.svelte';
	import Button from '$lib/ui/Button.svelte';

	let { id = $bindable(), value = $bindable() }: { id?: number; value: string } = $props();
	const { mutate: deleteModel } = DeleteModel();
	const { isLoading, data } = useModels();
</script>

{#if $isLoading}
	<div class="mb-4 flex items-center justify-center p-6 text-lg">Loading models...</div>
{:else if $data != undefined}
	<div class="grow space-y-2 overflow-y-auto">
		{#each $data.list as model (model.id)}
			<Button
				class="flex w-full flex-row items-center justify-between px-3 py-2"
				onclick={() => {
					id = model.id;
					value = 'openrouter_edit';
				}}
			>
				{model.display_name}
				<CheckDelete
					ondelete={() =>
						deleteModel({
							id: model.id
						})}
				/>
			</Button>
		{/each}
	</div>
{/if}
