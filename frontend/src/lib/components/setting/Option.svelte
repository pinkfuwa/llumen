<script lang="ts">
	import { ChevronDown } from '@lucide/svelte';
	import Select from '$lib/ui/Select.svelte';
	import { Collapsible } from 'bits-ui';
	import Radio from '$lib/ui/Radio.svelte';

	let {
		title,
		selected = $bindable<string>(''),
		data = [],
		disabled = false,
		onchange
	}: {
		title: string;
		selected?: string;
		data?: Array<{ value: string; label: string; disabled?: boolean }>;
		disabled?: boolean;
		onchange?: () => void;
	} = $props();
</script>

<div class="mb-4 hidden items-center justify-between border-b border-outline pb-2 text-lg md:flex">
	<span>{title}: </span>
	<Select bind:selected class="w-36 truncate" popupClass="w-38" {data} {disabled} {onchange} />
</div>

<Collapsible.Root class="md:hidden">
	<Collapsible.Trigger
		class="flex w-full flex-row flex-nowrap justify-between rounded p-2 text-lg duration-150 hover:bg-primary hover:text-text-hover"
	>
		<span>{title}</span>
		<ChevronDown />
	</Collapsible.Trigger>
	<Collapsible.Content
		class="flex flex-col border-b border-outline px-2 pt-2 slide-out-to-start-0 slide-in-from-top-0 fade-in fade-out data-[state=close]:animate-out data-[state=open]:animate-in"
	>
		<Radio {data} bind:selected {disabled} {onchange} />
	</Collapsible.Content>
</Collapsible.Root>
