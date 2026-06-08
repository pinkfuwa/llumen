<script lang="ts">
	import { LoaderCircle } from '@lucide/svelte';
	import Select from '$lib/ui/Select.svelte';
	import { models } from '$lib/api/model.svelte';
	import { Context } from '@sveltevietnam/i18n';
	import * as m from '@sveltevietnam/i18n/generated/messages';
	let lang = $derived(Context.get().lang);
	let {
		value = undefined as string | null | undefined,
		onchange = undefined as ((id: string) => void) | undefined,
		disabled = false
	}: {
		value?: string | null;
		onchange?: (id: string) => void;
		disabled?: boolean;
	} = $props();

	const data = $derived(models.val);

	let selectData = $derived(
		data?.map((x) => ({
			value: `${x.id}`,
			label: x.display_name
		}))
	);

	let localSelected = $state<string | undefined>();

	$effect(() => {
		localSelected = value ?? undefined;
	});

	function handleChange() {
		if (localSelected != null && onchange) onchange(localSelected);
	}
</script>

{#if selectData == undefined}
	<div
		class="inline-flex h-full grow cursor-not-allowed items-center justify-between rounded-lg border border-border
		px-3 text-center text-nowrap text-foreground duration-150 sm:w-64 sm:grow-0"
	>
		<span class="flex min-w-0 grow items-center justify-start truncate">
			<span> {m['common.loading'](lang)} </span>
			<LoaderCircle class="ml-2 inline-block animate-spin" />
		</span>
	</div>
{:else}
	<Select
		data={selectData}
		fallback={m['chat.select_model'](lang)}
		bind:selected={localSelected}
		onchange={handleChange}
		{disabled}
		class="grow truncate sm:w-64 sm:grow-0"
		popupClass="w-64"
	></Select>
{/if}
