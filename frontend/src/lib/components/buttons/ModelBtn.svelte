<script lang="ts">
	import { useModels } from '$lib/api/model';
	let { value = $bindable('0') } = $props();

	let { isLoading, data } = useModels();

	$effect(() => {
		if ($data) {
			let lastModel = $data?.at(-1);
			if (lastModel) value = lastModel.modelId;
		}
	});
</script>

{#if $data == undefined}
	<select
		class="min-w-[170px] items-center rounded-md bg-background px-3 py-1 hover:bg-hover"
		disabled
		required
	>
		<option>Loading...</option>
	</select>
{:else}
	<select
		class="min-w-[170px] items-center rounded-md bg-background px-3 py-1 hover:bg-hover"
		{value}
		required
	>
		{#each $data! as model}
			<option value={model.modelId}>{model.displayName}</option>
		{/each}
	</select>
{/if}
