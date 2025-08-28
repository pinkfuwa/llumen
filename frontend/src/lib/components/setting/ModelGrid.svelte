<script lang="ts">
	import { DeleteModel, useModels } from '$lib/api/model';
	import { Trash } from '@lucide/svelte';
	import { _ } from 'svelte-i18n';

	const { mutate: deleteModel } = DeleteModel();
	const { isLoading, data } = useModels();
</script>

{#if $isLoading}
	<div class="mb-4 flex items-center justify-center p-6 text-lg">Loading models...</div>
{:else if $data != undefined}
	<ul class="grid max-h-[50vh] grid-cols-1 gap-2 overflow-y-auto pb-2 text-lg lg:grid-cols-2">
		{#each $data.list as model}
			<li
				class="flex min-h-[50px] shrink-0 items-center justify-between rounded-lg border border-outline py-1 pr-2 pl-4"
			>
				{model.display_name}
				<button
					onclick={() =>
						deleteModel({
							id: model.id
						})}
				>
					<Trash class="h-10 w-10 rounded-lg p-2 hover:bg-hover" />
				</button>
			</li>
		{/each}
	</ul>
{/if}
