<script lang="ts">
	import { DeleteModel, useModels } from '$lib/api/model';
	import { Trash } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';
	import CheckDelete from './CheckDelete.svelte';

	const { mutate: deleteModel } = DeleteModel();
	const { isLoading, data } = useModels();
</script>

{#if $isLoading}
	<div class="mb-4 flex items-center justify-center p-6 text-lg">Loading models...</div>
{:else if $data != undefined}
	<div class="grid max-h-[50vh] grid-cols-1 gap-2 overflow-y-auto pb-2 text-lg lg:grid-cols-2">
		{#each $data.list as model}
			<div
				class="flex min-h-[50px] shrink-0 items-center justify-between rounded-lg border border-outline px-2 py-1"
			>
				<a
					class="flex h-full grow items-center rounded-md pl-2 hover:bg-hover"
					href={'/setting/openrouter/' + encodeURIComponent(model.id)}
				>
					{model.display_name}
				</a>
				<!-- TODO: mutation was supposed to be at top level -->
				<CheckDelete
					ondelete={deleteModel({
						id: model.id
					})}
				/>
			</div>
		{/each}
	</div>
{/if}
