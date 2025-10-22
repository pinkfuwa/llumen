<script lang="ts">
	import { ChevronDown } from '@lucide/svelte';
	import { Select } from 'bits-ui';
	import { cubicOut } from 'svelte/easing';
	import type { ClassValue } from 'svelte/elements';

	let {
		data = [],
		selected = $bindable(),
		fallback = '',
		class: className,
		popupClass,
		disabled = false,
		onchange = () => {}
	}: {
		data?: Array<{ value: string; label: string; disabled?: boolean }>;
		selected?: string;
		fallback?: string;
		class?: ClassValue;
		popupClass?: ClassValue;
		disabled?: boolean;
		onchange?: () => void;
	} = $props();

	const selectedLabel = $derived.by(() => {
		if (selected) {
			let item = data.find((data) => data.value == selected);
			if (item) {
				return item.label;
			}
		}
		return fallback;
	});

	function autoScrollDelay(tick: number) {
		const maxDelay = 200;
		const minDelay = 25;
		const steps = 30;

		const progress = Math.min(tick / steps, 1);
		// Use the cubicOut easing function from svelte/easing
		return maxDelay - (maxDelay - minDelay) * cubicOut(progress);
	}
</script>

<Select.Root
	type="single"
	onValueChange={(v) => {
		selected = v;
		onchange();
	}}
	items={data}
	{disabled}
>
	<Select.Trigger
		class="inline-flex h-full items-center justify-between rounded-lg border border-outline px-3 text-center text-nowrap text-text duration-150 not-disabled:cursor-pointer not-disabled:hover:bg-primary not-disabled:hover:text-text-hover focus:ring-4 focus:ring-outline focus:outline-none disabled:cursor-not-allowed {className}"
		{disabled}
	>
		<span class="flex min-w-0 grow justify-start truncate">
			{selectedLabel}
		</span>

		{#if !disabled}
			<ChevronDown class="inline-block shrink-0" />
		{/if}
	</Select.Trigger>
	<Select.Portal>
		<Select.Content
			class="z-50 max-h-48 rounded-xl border border-outline bg-input text-text outline-hidden select-none data-[side=bottom]:translate-y-1 data-[side=bottom]:slide-in-from-top-2 data-[side=top]:-translate-y-1 data-[side=top]:slide-in-from-bottom-2 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95 {popupClass} "
			sideOffset={10}
		>
			<Select.Viewport class="rounded-xl bg-clip-padding">
				{#each data as row, i}
					<Select.Item
						class="flex h-10 w-full items-center px-2 py-3 text-sm outline-hidden duration-150 select-none not-disabled:cursor-pointer not-disabled:hover:bg-primary not-disabled:hover:text-text-hover disabled:opacity-50"
						value={row.value}
						label={row.label}
						disabled={row.disabled}
					>
						{#snippet children({ selected })}
							{row.label}
							{#if selected}
								<div class="ml-auto h-1 w-1 bg-amber-200"></div>
							{/if}
						{/snippet}
					</Select.Item>
				{/each}
			</Select.Viewport>
		</Select.Content>
	</Select.Portal>
</Select.Root>
