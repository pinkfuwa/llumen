<script lang="ts">
	import { LoaderCircle } from '@lucide/svelte';
	import Select from '$lib/ui/Select.svelte';
	import { getContext } from 'svelte';
	import type { Readable } from 'svelte/store';
	import type { ModelListResp } from '$lib/api/types';
	import { _ } from 'svelte-i18n';
	let { value = $bindable<string | undefined>(), disabled = false } = $props();

	const data = getContext<Readable<ModelListResp | undefined>>('models');

	let selectData = $derived(
		$data?.list.map((x) => ({
			value: `${x.id}`,
			label: x.display_name
		}))
	);
</script>

{#if selectData == undefined}
	<div
		class="inline-flex h-full grow cursor-not-allowed items-center justify-between rounded-lg border border-outline
		px-3 text-center text-nowrap text-text duration-150 sm:w-56 sm:grow-0"
	>
		<span class="flex min-w-0 grow items-center justify-start truncate">
			<span> {$_('common.loading')} </span>
			<LoaderCircle class="ml-2 inline-block animate-spin" />
		</span>
	</div>
{:else}
	<Select
		data={selectData}
		fallback={$_('chat.select_model')}
		bind:selected={value}
		{disabled}
		class="grow truncate sm:w-56 sm:grow-0"
		popupClass="w-56"
	></Select>
{/if}
