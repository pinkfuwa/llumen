<script lang="ts">
	import { ChevronDown, LoaderCircle } from '@lucide/svelte';
	import { useModels } from '$lib/api/model';
	import Select from '$lib/ui/Select.svelte';
	import { number } from 'svelte-i18n';
	let { value = $bindable<string | undefined>(), above = false, disabled = false } = $props();

	let { data } = useModels();

	$effect(() => {
		$inspect(value);
		if (!disabled && $data) {
			let lastModel = $data?.list.at(-1);
			if (lastModel && value == undefined) {
				value = `${lastModel.id}`;
			} else if (value != undefined) {
				value = `${value}`;
			}
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
		class="flex items-center justify-between rounded-md border border-outline px-3 py-2 text-left font-mono"
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
		class="w-52"
		popupClass="w-52"
	></Select>
{/if}
