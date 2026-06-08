<script lang="ts">
	import { LoaderCircle } from '@lucide/svelte';
	import Select from '$lib/ui/Select.svelte';
	import { models } from '$lib/api/model.svelte';
	import { t } from 'svelte-intl-precompile';
	import { onModelChange, effective } from './state.svelte';

	let {
		disabled = false
	}: {
		disabled?: boolean;
	} = $props();

	const data = $derived(models.val);

	let selectData = $derived(
		data?.map((x) => ({
			value: `${x.id}`,
			label: x.display_name
		}))
	);

	let localSelected = $derived<string | undefined>(effective.modelId?.toString());

	function handleChange() {
		if (localSelected != null) onModelChange(localSelected);
	}
</script>

{#if selectData == undefined}
	<div
		class="inline-flex h-full grow cursor-not-allowed items-center justify-between rounded-lg border border-border
		px-3 text-center text-nowrap text-foreground duration-150 sm:w-64 sm:grow-0"
	>
		<span class="flex min-w-0 grow items-center justify-start truncate">
			<span> {$t('common.loading')} </span>
			<LoaderCircle class="ml-2 inline-block animate-spin" />
		</span>
	</div>
{:else}
	<Select
		data={selectData}
		fallback={$t('chat.select_model')}
		bind:selected={localSelected}
		onchange={handleChange}
		{disabled}
		class="grow truncate sm:w-64 sm:grow-0"
		popupClass="w-64"
	></Select>
{/if}
