<script lang="ts">
	import { useModels } from '$lib/api/model';
	let { model: value = $bindable('0') } = $props();

	let models = useModels();

	$effect(() => {
		if ($models.isFetched) {
			let lastModel = $models.data?.at(-1);
			console.log('set to', lastModel);
			if (lastModel) value = lastModel.modelId;
		}
	});
</script>

{#if $models.isFetched}
	<select
		class="min-w-[170px] items-center rounded-md bg-background px-3 py-1 hover:bg-hover"
		{value}
		required
	>
		{#each $models.data! as model, i}
			<option value={model.modelId}>{model.displayName}</option>
		{/each}
	</select>
{:else}
	<select
		class="min-w-[170px] items-center rounded-md bg-background px-3 py-1 hover:bg-hover"
		disabled
		required
	>
		<option>Loading...</option>
	</select>
{/if}
