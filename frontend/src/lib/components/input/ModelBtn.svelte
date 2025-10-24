<script lang="ts">
	import { LoaderCircle } from '@lucide/svelte';
	import { useModels } from '$lib/api/model';
	import Select from '$lib/ui/Select.svelte';
	import { getSupportedFileTypes } from './fileTypes';
	import { getContext, untrack } from 'svelte';
	import type { Readable, Writable } from 'svelte/store';
	import { lastModel } from '$lib/preference';
	import type { ModelListResp } from '$lib/api/types';
	let { value = $bindable<string | undefined>(), disabled = false } = $props();

	const filetypes = getContext<Writable<string>>('filetypes');

	const data = getContext<Readable<ModelListResp | undefined>>('models');

	$effect(() => {
		if (!disabled && $data) {
			let lastModelId = untrack(() => $lastModel);

			if (value == undefined) {
				const list = $data.list;
				let model = list.find((x) => x.id == lastModelId) || list.at(-1);

				if (model) value = model.id;
			}
		}
	});

	$effect(() => {
		let id: number | null = parseInt(value);
		if (!isNaN(id)) lastModel.set(id);
	});

	$effect(() => {
		let selectModelCap = $data?.list.find((x) => x.id == value);
		if (selectModelCap != undefined) {
			filetypes.set(getSupportedFileTypes(selectModelCap));
		}
	});

	let select_data = $derived(
		$data?.list.map((x) => ({
			value: `${x.id}`,
			label: x.display_name
		}))
	);
</script>

{#if select_data == undefined}
	<div
		class="flex min-w-0 items-center justify-between truncate rounded-md border border-outline px-3 py-2 text-left font-mono"
	>
		<span> Loading </span>
		<LoaderCircle class="inline-block animate-spin" />
	</div>
{:else}
	<Select
		data={select_data}
		fallback="Select Model"
		bind:selected={value}
		{disabled}
		class="grow truncate sm:w-56 sm:grow-0"
		popupClass="w-56"
	></Select>
{/if}
