<script lang="ts">
	import { DeleteModel, useModels } from '$lib/api/model';
	import { Trash } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import CheckDelete from './CheckDelete.svelte';
	import { Button } from 'bits-ui';

	let { id = $bindable(), value = $bindable() }: { id?: number; value: string } = $props();
	const { mutate: deleteModel } = DeleteModel();
	const { isLoading, data } = useModels();
</script>

{#if $isLoading}
	<div class="mb-4 flex items-center justify-center p-6 text-lg">Loading models...</div>
{:else if $data != undefined}
	<div class="grid max-h-[50vh] grid-cols-1 gap-2 overflow-y-auto pb-2 text-lg xl:grid-cols-2">
		{#each $data.list as model (model.id)}
			<div
				class="flex min-h-[50px] shrink-0 items-center justify-between rounded-lg border border-outline px-3 py-2 text-text hover:bg-primary hover:text-text-hover"
			>
				<Button.Root
					class="h-full w-full text-left"
					onclick={() => {
						id = model.id;
						value = 'openrouter_edit';
					}}
				>
					{model.display_name}
				</Button.Root>
				<!-- TODO: mutation was supposed to be at top level -->
				<CheckDelete
					ondelete={() =>
						deleteModel({
							id: model.id
						})}
				/>
			</div>
		{/each}
	</div>
{/if}
