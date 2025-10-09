<script lang="ts">
	import { LoaderCircle } from '@lucide/svelte';
	import { useModels } from '$lib/api/model';
	import Select from '$lib/ui/Select.svelte';
	import { getSupportedFileTypes } from './fileTypes';
	import { getContext } from 'svelte';
	import type { Writable } from 'svelte/store';
	let { value = $bindable<string | undefined>(), disabled = false } = $props();

	const filetypes = getContext<Writable<string>>('filetypes');

	let { data } = useModels();

	$effect(() => {
		if (!disabled && $data) {
			// TODO: select last selected model
			let lastModel = $data.list.at(-1);
			if (lastModel && value == undefined) {
				value = `${lastModel.id}`;
			} else if (value != undefined) {
				value = `${value}`;
			}
		}
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
