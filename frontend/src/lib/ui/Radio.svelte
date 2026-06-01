<script lang="ts">
	import { RadioGroup } from 'bits-ui';
	import type { ClassValue } from 'svelte/elements';

	let {
		data = [],
		selected = $bindable(),
		fallback = '',
		onchange = () => {}
	}: {
		data?: Array<{ value: string; label: string; disabled?: boolean }>;
		selected?: string;
		fallback?: string;
		disabled?: boolean;
		onchange?: () => void;
	} = $props();

	const itemStyle =
		'cursor-pointer rounded-lg p-2 text-left text-foreground duration-150 not-disabled:hover:bg-interactive-hover focus:ring-4 focus:ring-ring focus:outline-none disabled:cursor-not-allowed disabled:opacity-60 data-[state=checked]:bg-interactive-selection data-[state=checked]:text-primary';

	const selectedLabel = $derived.by(() => {
		if (selected) {
			let item = data.find((data) => data.value == selected);
			if (item) {
				return item.label;
			}
		}
		return fallback;
	});
</script>

<RadioGroup.Root bind:value={selected} class="flex flex-col">
	{#each data as entry}
		<RadioGroup.Item value={entry.value} class={itemStyle} onclick={onchange}
			>{entry.label}</RadioGroup.Item
		>
	{/each}
</RadioGroup.Root>
