<script lang="ts">
	import { ChevronDown } from '@lucide/svelte';
	import { useModels } from '$lib/api/model';
	let { value = $bindable('0') } = $props();

	let { data } = useModels();

	let open = $state(false);

	$effect(() => {
		if ($data) {
			let lastModel = $data?.at(-1);
			if (lastModel) value = lastModel.modelId;
		}
	});
</script>

{#if $data == undefined}
	<div
		class="flex h-[34px] min-w-[200px] items-center justify-between rounded-md bg-background py-[calc(0.25rem+1px)] pr-1 pl-3 text-left font-mono"
	>
		Loading...
	</div>
{:else}
	<div class="relative">
		<button
			class="min-w-[200px] items-center rounded-md {open
				? 'bg-hover'
				: 'bg-background'} flex justify-between py-[calc(0.25rem+1px)] pr-1 pl-3 text-left font-mono hover:bg-hover"
			onclick={() => {
				open = !open;
			}}
		>
			<span>
				{$data.find((x) => x.modelId == value)?.displayName}
			</span>
			<ChevronDown class="inline-block" />
		</button>
		{#if open}
			<ul
				class="absolute mt-1 min-w-[calc(100%+1rem)] space-x-4 rounded-md border border-outline bg-light p-2 font-mono"
			>
				{#each $data as model}
					<li
						class="text-md flex w-full items-center justify-between rounded-sm p-1.5 hover:bg-hover"
					>
						<button
							class="w-full text-left"
							onclick={() => {
								open = false;
								value = model.modelId;
							}}
						>
							{model.displayName}
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	</div>
{/if}
